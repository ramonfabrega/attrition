use super::*;
use crate::UnitType;
use crate::combat::Rng;
use crate::tuning::Tuning;
use crate::world::World;

/// **An unpack re-reads the crew's track, and a trackless figure
/// stands on its leader** (`Guy::update_gpiece@005d8530:52`–`:68`,
/// `cast_unpack@006709c0:60`–`:63`, item 1608, `docs/AI.md` §170). A
/// two-figure packer whose packed crew piece tracks at (67, −100) and
/// whose plain one names no track: packed, figure 1 stands on its
/// offset; unpacked, it has no [`Follow`] and shares guy 0's point;
/// packed again, it is back on the offset. The coverage pair's Advanced
/// Machine Gun `1/74` on block 2894. Made to fail with `update_gpiece`
/// leaving the follow alone.
#[test]
fn an_unpack_puts_a_trackless_crew_figure_on_its_leader() {
    let mut s = sim_at(1);
    let ty = s.add_unit_type(UnitType {
        hits: 100,
        ..UnitType::default()
    });
    s.unit_types[ty].combat.packs = true;
    // Two figures: `CREW_SIZE 1`, which `gpiece_walk` bounds the crew by.
    s.unit_types[ty].combat.crew_size = 1;
    s.unit_types[ty].type_index = 0x7f;
    let piece = |guy: i32, gender: bool| {
        0x7f - 0x32
            + FIRST_UNIT_PIECE
            + PIECES_PER_CREW * guy
            + if gender { PIECES_PER_GENDER } else { 0 }
    };
    for guy in 0..2 {
        for gender in [false, true] {
            s.art
                .piece_lengths
                .insert(piece(guy, gender), BTreeMap::new());
        }
    }
    s.art.tracks.insert(piece(1, true), (67, -100));
    let at = Pos::new(0x4000, 0x4000);
    let mut unit = Unit::new(1, 3, at, 100);
    unit.ty = Some(ty);
    unit.on_map = true;
    let u = s.add_unit(unit);
    s.units[u].guys = vec![Guy::fresh(-1), Guy::fresh(-1)];
    s.units[u].movement.body.pos = at;
    s.units[u].combat.packed = true;
    s.update_gpiece(u);
    s.set_new_location(u, at, true);
    let offset = s.units[u].guys[1].follow.expect("a tracked crew figure");
    assert_eq!(offset.track, (67, -100));
    assert_ne!(offset.body.pos, at, "packed, it stands on its offset");
    s.cast_unpack(u);
    assert!(!s.units[u].combat.packed);
    assert_eq!(s.units[u].guys[1].gpiece, piece(1, false));
    assert_eq!(
        s.units[u].guys[1].follow, None,
        "unpacked, the piece names no track: the figure is on its leader"
    );
    s.cast_pack(u);
    let back = s.units[u].guys[1]
        .follow
        .expect("packed again, tracked again");
    assert_eq!(back.track, (67, -100));
    assert_eq!(
        back.body.pos, offset.body.pos,
        "and snapped onto its offset"
    );
}

/// **The unpack's re-seat** (`cast_unpack@006709c0:60`–`:63`, item
/// 1608): a crew figure whose unpacked piece still tracks, at another
/// offset, is put on the new offset the frame it unpacks rather than
/// walking there. No capture holds it — the machine gun's unpacked
/// crew has no track, and stands on its leader either way — so this
/// test is the arm's only holder. Made to fail with the re-seat
/// removed.
#[test]
fn an_unpack_puts_a_tracked_crew_figure_on_its_new_offset() {
    let mut s = sim_at(1);
    let ty = s.add_unit_type(UnitType {
        hits: 100,
        ..UnitType::default()
    });
    s.unit_types[ty].combat.packs = true;
    // Two figures: `CREW_SIZE 1`, which `gpiece_walk` bounds the crew by.
    s.unit_types[ty].combat.crew_size = 1;
    s.unit_types[ty].type_index = 0x7f;
    let piece = |guy: i32, gender: bool| {
        0x7f - 0x32
            + FIRST_UNIT_PIECE
            + PIECES_PER_CREW * guy
            + if gender { PIECES_PER_GENDER } else { 0 }
    };
    for guy in 0..2 {
        for gender in [false, true] {
            s.art
                .piece_lengths
                .insert(piece(guy, gender), BTreeMap::new());
        }
    }
    s.art.tracks.insert(piece(1, true), (67, -100));
    s.art.tracks.insert(piece(1, false), (0, -192));
    let at = Pos::new(0x4000, 0x4000);
    let mut unit = Unit::new(1, 3, at, 100);
    unit.ty = Some(ty);
    unit.on_map = true;
    let u = s.add_unit(unit);
    s.units[u].guys = vec![Guy::fresh(-1), Guy::fresh(-1)];
    s.units[u].movement.body.pos = at;
    s.units[u].combat.packed = true;
    s.update_gpiece(u);
    s.set_new_location(u, at, true);
    let packed = s.units[u].guys[1].follow.expect("tracked").body.pos;
    s.cast_unpack(u);
    let f = s.units[u].guys[1].follow.expect("still tracked");
    assert_eq!(f.track, (0, -192));
    assert_ne!(f.des, packed, "the new offset is elsewhere");
    assert_eq!(f.body.pos, f.des, "and the figure is put on it");
}

/// **The stall arm's table** (`get_unit_gpiece@0090c030:104`–`:140`;
/// item 3369): each build style's piece, the age gate (bracket 1, or 0
/// for style 6 alone), and the styles that take none.
#[test]
fn an_unpacked_merchant_s_over_time_piece_is_its_build_style_s() {
    assert_eq!(
        stall_piece(1, 1),
        Some(TOTAL_UNIT_PIECES - 5),
        "NEUROPE-AGE3, 50689"
    );
    assert_eq!(stall_piece(1, 1), Some(50689));
    assert_eq!(
        stall_piece(3, 1),
        Some(TOTAL_UNIT_PIECES - 6),
        "KOREAN-AGE3"
    );
    assert_eq!(
        stall_piece(6, 0),
        Some(TOTAL_UNIT_PIECES - 4),
        "IROQUOIS-AGE0"
    );
    assert_eq!(
        stall_piece(6, 1),
        Some(TOTAL_UNIT_PIECES - 3),
        "IROQUOIS-AGE3"
    );
    assert_eq!(
        stall_piece(7, 1),
        Some(TOTAL_UNIT_PIECES - 2),
        "COLONIAL-AGE3"
    );
    assert_eq!(
        stall_piece(8, 1),
        Some(TOTAL_UNIT_PIECES - 1),
        "EINDIAN-AGE3"
    );
    assert_eq!(stall_piece(1, 0), None, "the first bracket is the walks'");
    assert_eq!(stall_piece(1, 2), None, "and so is the last");
    assert_eq!(stall_piece(7, 0), None, "only style 6 takes bracket 0");
    for build in [0, 2, 4, 5] {
        assert_eq!(stall_piece(build, 1), None, "build style {build} has none");
    }
}

fn stepped(seed: u32, n: usize) -> u32 {
    let mut r = Rng::new(seed);
    for _ in 0..n {
        r.roll();
    }
    r.seed
}

fn sim_at(seed: u32) -> Sim {
    let mut s = Sim::new(Tuning::RON, World::new(60, 60), 2);
    s.rng = Rng::new(seed);
    s
}

fn animal(s: &mut Sim, who: Player, o: i16, piece: i32, anim: i8, cur: u32, end: u32) -> usize {
    let mut u = Unit::new(who, o, Pos::new(3000, 3000), 10);
    u.guys = vec![Guy {
        cur_time: cur,
        end_time: end,
        last_time: -1,
        anim,
        gpiece: piece,
        flag_piece: piece,
        stopped: true,
        pending_attack: 0,
        queued_attack: 0,
        aim: None,
        follow: None,
        turret: Turret::ZERO,
    }];
    s.add_unit(u)
}

/// **A pushed unit's trackless crew arrives where its guy 0 does not**
/// (`docs/COLLISION.md` §16). A push rewrites guy 0's destination
/// (`Unit::set_new_location`) and leaves the crew's, which is guy 0's
/// old point and the one the shared body still stands on. So a
/// blocked stand asked before the figures catch up passes guy 0's
/// walking early return and rolls the crew's idle: one draw, not two
/// and not none. Golden chapter eleven's chariot on tick 734.
///
/// Written to fail first: with the crew answering guy 0's arrival test
/// the request draws nothing.
#[test]
fn a_pushed_unit_s_trackless_crew_rolls_its_idle_alone() {
    let mut s = sim_at(12345);
    let u = animal(&mut s, 0, 1, -1, WALK, 2, 30);
    let crew = s.units[u].guys[0];
    s.units[u].guys.push(crew);
    // Pushed: the unit is on a new point, the body on the old one.
    s.units[u].movement.body.pos = Pos::new(2957, 2979);
    let before = s.rng.seed;
    s.set_default_anim(u);
    assert_ne!(s.rng.seed, before, "the crew's idle was not rolled");
    assert_eq!(category(s.units[u].guys[0].anim), 8, "guy 0 left its walk");
    assert_eq!(
        category(s.units[u].guys[1].anim),
        0,
        "the crew kept its walk"
    );
    let mut once = Rng::new(before);
    once.roll();
    assert_eq!(s.rng.seed, once.seed, "more than one draw");
}

/// **An unpacked packer's moving figure asks for no walk**
/// (`Guy::move:176–181`, item 625). A figure whose type packs
/// (`unit_flags2 & 4`) takes `Guy::move`'s moving arm without a
/// `set_anim` while the unit is unpacked, and asks for the walk as
/// any other once it is packed. Golden chapter three's restage is the
/// first case: the catapult's crew, pulled round by its turn in place.
///
/// Written to fail first: with the gate off the unpacked figure takes
/// `CHAR_WALK` on the first call.
#[test]
fn an_unpacked_packer_s_moving_figure_asks_for_no_walk() {
    let mut s = sim_at(12345);
    let t = s.add_unit_type(crate::UnitType::default());
    s.unit_types[t].combat.packs = true;
    let u = animal(&mut s, 0, 6, -1, DEFAULT, 5, 79);
    s.units[u].ty = Some(t);
    s.units[u].combat.packed = false;
    let before = s.rng.seed;
    s.guy_follow_anim(u, 0, false, true);
    assert_eq!(
        (
            s.units[u].guys[0].anim,
            s.units[u].guys[0].cur_time,
            s.rng.seed
        ),
        (DEFAULT, 5, before),
        "an unpacked packer's moving figure asked for the walk"
    );
    s.units[u].combat.packed = true;
    s.guy_follow_anim(u, 0, false, true);
    assert_eq!(
        category(s.units[u].guys[0].anim),
        category(WALK),
        "a packed packer's moving figure did not walk"
    );
}

/// **A moving figure on a turn slot is put back on the walk**
/// (`Guy::move:176–181`, the listing `5d9565`–`5d958a`; item 3360).
/// The moving arm's gate reads the plane bit and the pack pair and
/// nothing of `cur_anim`, so a push turn asked for earlier in the frame
/// does not survive a frame the body moves. The Inca word's Trebuchet
/// `1/118` on 16512 (`docs/ANIM.md` §16).
///
/// Written to fail first: with the turn slots spared the figure keeps
/// `CHAR_TURN_RIGHT`.
#[test]
fn a_moving_figure_on_a_turn_slot_is_put_back_on_the_walk() {
    let mut s = sim_at(12345);
    for turn in [TURN_LEFT, TURN_RIGHT] {
        let u = animal(&mut s, 0, 6, -1, turn, 1, 20);
        s.guy_follow_anim(u, 0, false, false);
        assert_eq!(
            category(s.units[u].guys[0].anim),
            category(WALK),
            "a moving figure kept its turn slot {turn}"
        );
    }
}

/// **An attack asked of a guy that is still turning costs no draw and
/// is not lost** (`docs/ANIM.md` §6.2). The three states the byte can
/// be in, each asserted against the stream's own word: deferred (no
/// draw), still deferred on a turning frame (no draw, debt kept), and
/// paid on the frame the angle settles (one draw).
///
/// Written to fail first: with the debt cleared on the turning frame
/// the third assertion sees no draw at all, which is exactly the one
/// the golden record is missing at 617.
#[test]
fn an_attack_asked_of_a_turning_guy_is_deferred_and_then_paid() {
    let mut s = sim_at(12345);
    let u = animal(&mut s, 0, 1, -1, DEFAULT, 0, 90);
    // The swing: the heading moves, the facing has not caught up.
    s.units[u].movement.heading = crate::movement::Angle::EAST;
    s.units[u].movement.facing = crate::movement::Angle::NORTH;
    s.units[u].movement.body.pos = s.units[u].pos;
    let before = s.rng.seed;
    s.set_anim(u, ATTACK1, false, true);
    assert_eq!(
        s.rng.seed, before,
        "the deferred attack drew from the stream"
    );
    assert_eq!(s.units[u].guys[0].pending_attack, 1);
    assert_eq!(
        s.units[u].guys[0].anim, DEFAULT,
        "the attack was played, not deferred"
    );

    // A frame still owed its turn: `Guy::move`'s other arm re-enters
    // `set_anim`, which defers again — no draw, and the debt stands.
    s.guy_follow_anim(u, 0, true, false);
    assert_eq!(s.rng.seed, before, "the turning frame drew from the stream");
    assert_eq!(
        s.units[u].guys[0].pending_attack, 1,
        "the turning arm cleared a debt it could not pay"
    );

    // The frame the angle settles: one draw, and an attack slot.
    s.units[u].movement.facing = crate::movement::Angle::EAST;
    s.guy_follow_anim(u, 0, true, true);
    assert_eq!(
        s.rng.seed,
        stepped(before, 1),
        "the settled arm did not spend the attack roll"
    );
    assert_eq!(s.units[u].guys[0].pending_attack, 0);
    assert_eq!(category(s.units[u].guys[0].anim), 12);
    assert!(s.units[u].guys[0].stopped);
}

/// **An attack asked of a guy already swinging is queued, not
/// played** — `GuyData +0xa0`, and it costs no draw either
/// (`docs/ANIM.md` §6.2).
#[test]
fn an_attack_asked_mid_swing_is_queued() {
    let mut s = sim_at(999);
    let u = animal(&mut s, 0, 1, -1, ATTACK2, 0, 20);
    s.units[u].movement.heading = crate::movement::Angle::NORTH;
    s.units[u].movement.facing = crate::movement::Angle::NORTH;
    s.units[u].movement.body.pos = s.units[u].pos;
    let before = s.rng.seed;
    // `Unit::set_anim`'s pre-clear must **not** fire: guy 0 is already
    // on an attack, which is the whole point of the queue.
    s.set_anim(u, ATTACK1, false, true);
    assert_eq!(s.rng.seed, before, "the queued attack drew from the stream");
    assert_eq!(s.units[u].guys[0].queued_attack, 1);
    assert_eq!(s.units[u].guys[0].pending_attack, 0);
    assert_eq!(
        s.units[u].guys[0].anim, ATTACK2,
        "the queued attack was played"
    );
}

/// **Any unit-level request made off an attack cancels the debt** —
/// `Unit::set_anim@00616f40`'s own first statement, which is why a
/// squad member that walks on the frame it swings never plays its
/// attack (`docs/ANIM.md` §6.2). The original's who=1 hoplites are
/// two such and one that stands still; the one that stands is the
/// golden record's draw at 617.
#[test]
fn a_walk_request_cancels_the_attack_a_guy_owed() {
    let mut s = sim_at(4242);
    let u = animal(&mut s, 0, 1, -1, DEFAULT, 0, 90);
    s.units[u].movement.heading = crate::movement::Angle::EAST;
    s.units[u].movement.facing = crate::movement::Angle::NORTH;
    s.units[u].movement.body.pos = s.units[u].pos;
    s.set_anim(u, ATTACK1, false, true);
    assert_eq!(s.units[u].guys[0].pending_attack, 1);
    s.set_anim(u, WALK, false, true);
    assert_eq!(
        s.units[u].guys[0].pending_attack, 0,
        "the walk left the attack owing"
    );
}

/// **A piece the install describes has its whole slot list**, and a
/// slot it leaves out is the packet's own — three frames from
/// `AnimationPacket::get_game_frames`, and a variant the idle roll has
/// to fall back from. That is not the same question as "does anything
/// know a length", and reading it as one is what made the roll's
/// fallback swallow the human scout's `CHAR_IDLE1` (`docs/ANIM.md`
/// §3.2).
#[test]
fn a_known_piece_s_missing_slot_is_three_frames_and_not_a_variant() {
    let mut s = sim_at(1);
    let u = animal(&mut s, 0, 0, 7, DEFAULT, 0, 61);
    // Nothing knows the piece: every slot is unknown, and an unknown
    // length reads as a slot the packet lacks — which is what this
    // crate did everywhere before the install was read.
    assert_eq!(s.slot_length(u, 7, IDLE1), None);
    assert!(!s.packet_has(u, 7, IDLE1));
    // The install names it: `CHAR_DEFAULT` and `CHAR_IDLE1` only.
    s.art
        .piece_lengths
        .insert(7, [(DEFAULT, 61u32), (IDLE1, 76)].into_iter().collect());
    assert_eq!(s.slot_length(u, 7, IDLE1), Some(76));
    assert!(s.packet_has(u, 7, IDLE1));
    assert_eq!(s.slot_length(u, 7, IDLE2), Some(MISSING));
    assert!(!s.packet_has(u, 7, IDLE2));
    // And it wins over a dump row for the same pair.
    s.art.lengths.insert((7, IDLE1), 61);
    assert_eq!(s.slot_length(u, 7, IDLE1), Some(76));
}

/// **A rolled attack slot the packet lacks plays `CHAR_ATTACK2`** —
/// `Guy::set_anim@005da300`'s `cmove` at `0x5db279` (`docs/ANIM.md`
/// §4.13). The Trireme's packet names `ATTACK1` and `ATTACK2` only;
/// a roll in the `ATTACK3` band played three frames here and ran
/// out on golden chapter five's 739, a draw the original never
/// spent. Written to fail first: without the fallback the slot is
/// `ATTACK3` and the length [`MISSING`].
#[test]
fn a_rolled_attack_the_packet_lacks_plays_attack2() {
    let seed = (1u32..)
        .take(10_000)
        .find(|&sd| Rng::new(sd).roll() % 100 > 70)
        .expect("a seed in ATTACK3's band");
    // The control is a packet that names `ATTACK3`: the roll's slot
    // stands, so the fallback is the packet's and not the band's.
    for (names_attack3, slot, end) in [(false, ATTACK2, 40u32), (true, ATTACK3, 27)] {
        let mut s = sim_at(seed);
        let u = animal(&mut s, 1, 6, 290, DEFAULT, 0, 61);
        let mut slots: std::collections::BTreeMap<i8, u32> =
            [(DEFAULT, 61u32), (ATTACK1, 40), (ATTACK2, 40)]
                .into_iter()
                .collect();
        if names_attack3 {
            slots.insert(ATTACK3, 27);
        }
        s.art.piece_lengths.insert(290, slots.into_iter().collect());
        s.guy_set_anim(u, 0, ATTACK1, false, true);
        assert_eq!(s.rng.seed, stepped(seed, 1), "the attack roll is one draw");
        let g = s.units[u].guys[0];
        assert_eq!(
            (g.anim, g.end_time),
            (slot, end),
            "packet names ATTACK3: {names_attack3}; the slot the roll plays and its length"
        );
    }
}

/// **The scout's fifteen frames.** Run33's human scout wraps its idle
/// on frame 284 and the roll takes `CHAR_IDLE1`; the original's clock
/// then runs 76 frames and this crate's ran 61, because the variant
/// fell back to `CHAR_DEFAULT` for want of a length. The draw is the
/// same either way — what changed is the slot it lands on and the
/// frame the clock next wraps (`docs/ANIM.md` §3.2).
#[test]
fn the_scout_s_idle1_runs_seventy_six_frames_not_the_default_s_sixty_one() {
    // A word whose `% 100` is `IDLE1`'s band.
    let seed = (1u32..)
        .take(10_000)
        .find(|&sd| (70..=82).contains(&(Rng::new(sd).roll() % 100)))
        .expect("a seed in the band");
    for (known, wrap) in [(false, 345u32), (true, 360)] {
        let mut s = sim_at(seed);
        let u = animal(&mut s, 0, 0, 371, DEFAULT, 61, 61);
        s.art.lengths.insert((371, DEFAULT), 61);
        if known {
            s.art.piece_lengths.insert(
                371,
                [(DEFAULT, 61u32), (IDLE1, 76), (IDLE2, 41), (IDLE3, 190)]
                    .into_iter()
                    .collect(),
            );
        }
        s.guy_set_anim(u, 0, DEFAULT, false, true);
        let g = s.units[u].guys[0];
        assert_eq!(g.anim, if known { IDLE1 } else { DEFAULT });
        assert_eq!(g.end_time, if known { 76 } else { 61 });
        assert_eq!(284 + g.end_time, wrap, "the frame the clock next wraps");
    }
}

/// The categories as the executable holds them: idles 0, walks 8, the
/// carrying walks 8, the sow its own.
#[test]
fn the_categories() {
    assert_eq!(category(IDLE3), 0);
    assert_eq!(category(GROUP_IDLE3), 0);
    assert_eq!(category(SLOG), 8);
    assert_eq!(category(JOG), 8);
    assert_eq!(category(WALK_TO_WOOD), 8);
    assert_eq!(category(WALK_WITH_ORE), 8);
    assert_eq!(category(ATTACK1), 12);
    assert_eq!(category(ATTACK3), 12);
    assert_eq!(category(CHOP_WOOD), 25);
    assert_eq!(category(SOW), 35);
    assert_eq!(category(FARM), 37);
    assert_eq!(category(40), 0);
}

/// Run13, sim-frame 100: the twelve `HERDFISH` on piece 60073 (length
/// 101) reach 101 together in `o` order and each rolls one `% 100`
/// from `0x60032f25`, the word at the end of 99 — the replay gives the
/// variants `2,0,0,1,0,0,0,0,0,0,0,0`, which are the dump's new
/// `cur_anim`s (`docs/SYNC.md` §4.1).
#[test]
fn run13_s_fish_wrap_together() {
    let mut s = sim_at(0x6003_2f25);
    for (p, a) in [(60073, 0), (60074, 1), (60072, 2)] {
        s.art.lengths.insert((p, a), 0);
    }
    for a in 0..4 {
        s.art.lengths.insert((60073, a), 101);
        s.art.lengths.insert((60074, a), 116);
        s.art.lengths.insert((60072, a), 170);
    }
    let mut fish = Vec::new();
    for o in 4..=37 {
        let (piece, end) = match o % 3 {
            1 => (60073, 101),
            2 => (60074, 116),
            _ => (60072, 170),
        };
        let cur = 100;
        fish.push((o, animal(&mut s, 8, o, piece, 0, cur, end)));
    }
    s.frame = 100;
    s.guys_inc_time();
    let got: Vec<i8> = fish
        .iter()
        .filter(|(o, _)| o % 3 == 1)
        .map(|&(_, u)| s.units[u].guys[0].anim)
        .collect();
    assert_eq!(got, vec![2, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    for &(o, u) in &fish {
        let g = s.units[u].guys[0];
        if o % 3 == 1 {
            assert_eq!((g.cur_time, g.end_time, g.last_time), (0, 101, -1), "o {o}");
        } else {
            assert_eq!(g.cur_time, 101, "o {o} keeps counting");
        }
    }
    assert_eq!(s.rng.seed, stepped(0x6003_2f25, 12));
}

/// Run12, frame 0, draws 48–87: forty animals fresh from `init_real`
/// (`end_time 0`) each roll their first idle from `set_anim` on their
/// first `do_idle`, in `o` order — the variants the dump's end-of-frame
/// state shows at offset 48 and nowhere else. The four sheep are
/// pieces 60063/4/5 by `(seed + o) % 3` with seed 7236, the fish
/// 60072/3/4; the lengths are the dump's.
#[test]
fn run12_s_forty_first_idles() {
    let mut s = sim_at(stepped(0x3bd3_9ae9, 48));
    s.game_seed = 7236;
    for a in 0..4 {
        s.art.lengths.insert((60063, a), 90);
        s.art.lengths.insert((60064, a), 109);
        s.art.lengths.insert((60065, a), 250);
        s.art.lengths.insert((60073, a), 101);
        s.art.lengths.insert((60074, a), 116);
        s.art.lengths.insert((60072, a), 170);
    }
    let mut units = Vec::new();
    for o in 0..40i16 {
        let sheep = o < 4;
        let base = if sheep { 60063 } else { 60072 };
        let piece = base + (7236 + i32::from(o)) % 3;
        let end = 0;
        let u = animal(&mut s, 8, o, piece, 0, 0, end);
        units.push(u);
    }
    for &u in &units {
        s.set_default_anim(u);
    }
    assert_eq!(s.rng.seed, stepped(0x3bd3_9ae9, 88), "forty draws");
    // The dump's end-of-frame-0 `cur_anim` for `o` 0–39, read from
    // run12 with `tools/gamelog/anims.py` — and the LCG's rolls 48–87
    // give exactly these under the idle thresholds.
    let got: Vec<i8> = units.iter().map(|&u| s.units[u].guys[0].anim).collect();
    assert_eq!(
        got,
        vec![
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 3, 0, 2, 1, 0, 2, 0, 0, 0,
            0, 0, 1, 1, 3, 0, 3, 2, 1, 0, 0
        ]
    );
    assert_eq!(s.units[units[0]].guys[0].end_time, 90);
    assert_eq!(s.units[units[1]].guys[0].end_time, 109);
    assert_eq!(s.units[units[2]].guys[0].end_time, 250);
    assert_eq!(s.units[units[5]].guys[0].end_time, 116);
    assert_eq!(s.units[units[6]].guys[0].end_time, 170);
}

/// A walking guy asked to idle draws nothing until the body arrives,
/// then one draw starts the idle at 0 — the sheep at run13's 101.
#[test]
fn the_arrival_draws_once() {
    let mut s = sim_at(7);
    s.art.lengths.insert((60063, SLOG), 16);
    s.art.lengths.insert((60063, DEFAULT), 90);
    let u = animal(&mut s, 8, 0, 60063, SLOG, 11, 16);
    s.units[u].movement.body.pos = Pos::new(2990, 3000);
    let before = s.rng.seed;
    s.set_default_anim(u);
    assert_eq!(s.rng.seed, before, "still walking: no draw");
    assert_eq!(s.units[u].guys[0].anim, SLOG);
    s.units[u].movement.body.pos = Pos::new(3000, 3000);
    s.set_default_anim(u);
    assert_eq!(s.rng.seed, stepped(before, 1));
    let g = s.units[u].guys[0];
    assert_eq!((g.anim, g.cur_time, g.end_time), (DEFAULT, 0, 90));
}

/// **The standing body still owed a turn walks in place** —
/// `Guy::move@005d9240:73–89`, the arm the arrival's *second* draw
/// comes from (`docs/SYNC.md` §3.11, "the pair").
///
/// The body is on its unit, so the moving half never runs; but
/// `des_angle != angle`, so instead of idling it is put back on
/// `CHAR_WALK` and marked unstopped. The frame after, `do_idle` sees
/// the walk category and a stopped body again and rolls a second
/// time — which is why a farm animal arriving spends two
/// `Animal::do_idle` draws on consecutive frames, run39's 19 and 20.
///
/// The arm's only guard is a **sea** unit or a `SPECIAL_ANIM` order,
/// and the walk it asks for is the plain one: the carrying walk is
/// chosen in the moving half, which this is not.
///
/// Made to fail by returning early when the facing has not settled —
/// the body then idles on the arrival frame and draws once, not twice.
#[test]
fn a_standing_body_still_owed_a_turn_walks_in_place_and_draws_again() {
    let mut s = sim_at(7);
    s.art.lengths.insert((60063, WALK), 20);
    s.art.lengths.insert((60063, DEFAULT), 90);
    let u = animal(&mut s, 8, 0, 60063, WALK, 11, 20);
    // On its unit — the body has arrived — but a turn is still owed.
    s.units[u].movement.body.pos = s.units[u].pos;
    s.units[u].movement.facing = crate::movement::Angle::NORTH;
    s.units[u].movement.heading = crate::movement::Angle::EAST;
    s.guys_follow(u, true, &mut None);
    assert_eq!(s.units[u].guys[0].anim, WALK, "back on the walk");
    assert!(!s.units[u].guys[0].stopped, "and unstopped");

    // With the facing settled it is an arrival instead: one draw, and
    // the body stops.
    let mut s = sim_at(7);
    s.art.lengths.insert((60063, WALK), 20);
    s.art.lengths.insert((60063, DEFAULT), 90);
    let u = animal(&mut s, 8, 0, 60063, WALK, 11, 20);
    s.units[u].movement.body.pos = s.units[u].pos;
    let before = s.rng.seed;
    s.guys_follow(u, true, &mut None);
    assert_eq!(s.rng.seed, stepped(before, 1), "the arrival's own draw");
    assert_eq!(s.units[u].guys[0].anim, DEFAULT);
    assert!(s.units[u].guys[0].stopped);

    // A ship owed a turn takes the guard: stopped, and no walk.
    let mut s = sim_at(7);
    s.art.lengths.insert((60063, WALK), 20);
    let u = animal(&mut s, 8, 0, 60063, DEFAULT, 11, 90);
    s.units[u].kind.domain = crate::attrition::Domain::Sea;
    s.units[u].movement.body.pos = s.units[u].pos;
    s.units[u].movement.facing = crate::movement::Angle::NORTH;
    s.units[u].movement.heading = crate::movement::Angle::EAST;
    s.units[u].guys[0].stopped = false;
    s.guys_follow(u, true, &mut None);
    assert_eq!(s.units[u].guys[0].anim, DEFAULT, "a ship keeps its slot");
    assert!(s.units[u].guys[0].stopped);
}

/// The step: one a frame, `last_time` trailing; a work animation that
/// runs out restarts silently, keeping the overrun.
#[test]
fn the_step_and_a_silent_restart() {
    let mut s = sim_at(7);
    s.art.lengths.insert((6688, SOW), 47);
    let u = animal(&mut s, 0, 3, 6688, SOW, 45, 47);
    let before = s.rng.seed;
    s.guys_inc_time();
    assert_eq!(
        (s.units[u].guys[0].cur_time, s.units[u].guys[0].last_time),
        (46, 45)
    );
    s.guys_inc_time();
    let g = s.units[u].guys[0];
    assert_eq!((g.anim, g.cur_time, g.last_time), (SOW, 0, -1));
    assert_eq!(s.rng.seed, before, "the sow loops without a draw");
}

/// A second guy past the squad's size mirrors guy 0 and never draws —
/// the human scout's two guys wrap on one draw (run13's 101).
#[test]
fn the_mirror() {
    let mut s = sim_at(0xa45f_ecaf);
    s.art.lengths.insert((371, DEFAULT), 61);
    s.art.lengths.insert((13043, DEFAULT), 61);
    let u = animal(&mut s, 0, 0, 371, DEFAULT, 60, 61);
    s.units[u].guys.push(Guy {
        cur_time: 60,
        end_time: 61,
        last_time: -1,
        anim: DEFAULT,
        gpiece: 13043,
        flag_piece: 13043,
        stopped: true,
        pending_attack: 0,
        queued_attack: 0,
        aim: None,
        follow: None,
        turret: Turret::ZERO,
    });
    s.frame = 101;
    s.guys_inc_time();
    assert_eq!(s.rng.seed, stepped(0xa45f_ecaf, 1));
    let (a, b) = (s.units[u].guys[0], s.units[u].guys[1]);
    assert_eq!((a.cur_time, a.anim), (0, b.anim));
    assert_eq!(b.cur_time, 0);
    assert_eq!(b.last_time, -1);
}

/// **`guy_flags & 0x20` is guy 0's; the crew keeps four idle
/// variants** (`docs/ANIM.md` §13). `Unit::set_in_danger@005fcfb0`
/// sets the bit on figures `0 .. guy_mark` alone (`5fd01e`, the
/// image's one writer), so the crew figure past the squad rolls
/// without it. Golden chapter eleven's chariot, the guard `0/6`, with
/// its two pieces' lengths: on tick 1100 its crew's p91 is `IDLE2`, 71
/// frames, where the flag would give `IDLE1`, 81, and the re-roll on
/// 1139 is the one this crate missed.
///
/// Made to fail on purpose: with the flag read for every figure, the
/// crew takes `IDLE1`.
#[test]
fn a_crew_figure_rolls_its_idle_without_the_danger_flag() {
    // Two rolls in 83..=95 in a row: guy 0's, then the crew's.
    let seed = (1u32..)
        .find(|&x| {
            let mut r = Rng::new(x);
            let (a, b) = (r.roll() % 100, r.roll() % 100);
            (83..=95).contains(&a) && (83..=95).contains(&b)
        })
        .unwrap();
    let mut s = sim_at(seed);
    for (piece, lens) in [(145, [31, 80, 70, 31]), (12817, [31, 81, 71, 31])] {
        for (slot, len) in [DEFAULT, IDLE1, IDLE2, IDLE3].into_iter().zip(lens) {
            s.art.lengths.insert((piece, slot), len);
        }
    }
    let u = animal(&mut s, 0, 6, 145, IDLE1, 80, 80);
    let mut crew = s.units[u].guys[0];
    (crew.gpiece, crew.end_time) = (12817, 80);
    s.units[u].guys.push(crew);
    s.units[u].guy_flag_0x20 = true;
    s.set_default_anim(u);
    assert_eq!(s.rng.seed, stepped(seed, 2), "one roll a figure");
    let (a, b) = (s.units[u].guys[0], s.units[u].guys[1]);
    assert_eq!((a.anim, a.end_time), (IDLE1, 80), "guy 0 is flagged");
    assert_eq!((b.anim, b.end_time), (IDLE2, 71), "the crew is not");
}

/// **A unit holding a suspended search idles without a draw**
/// (`docs/ANIM.md` §14). `Guy::set_anim`'s roll is `openlist == 0 ?
/// rand % 100 : CHAR_DEFAULT` (`5dac5f`), so a wrap on a unit whose
/// search suspended plays `DEFAULT` and leaves the stream alone. East
/// Indies' 18999: `1/67`, suspended since 18969 (`start_dist 480`,
/// `collide 31` on run257's block 19000), wraps its idle beside four
/// others, and the original spends four draws where this crate spent
/// five.
#[test]
fn a_suspended_search_idles_without_a_draw() {
    // A roll that would take `IDLE3` if it were spent.
    let seed = (1u32..).find(|&x| Rng::new(x).roll() % 100 > 95).unwrap();
    let mut s = sim_at(seed);
    for (slot, len) in [(DEFAULT, 31), (IDLE1, 80), (IDLE2, 70), (IDLE3, 40)] {
        s.art.lengths.insert((145, slot), len);
    }
    let u = animal(&mut s, 0, 67, 145, DEFAULT, 31, 31);
    s.units[u].search = Some(Box::new(crate::path::Search::suspended_for_test()));
    s.set_default_anim(u);
    assert_eq!(s.rng.seed, seed, "no draw while the search is suspended");
    let g = s.units[u].guys[0];
    assert_eq!((g.anim, g.cur_time, g.end_time), (DEFAULT, 0, 31));
    // The same wrap with the stash freed rolls, and the roll is `IDLE3`.
    s.units[u].search = None;
    s.units[u].guys[0].cur_time = 31;
    s.set_default_anim(u);
    assert_eq!(s.rng.seed, stepped(seed, 1), "one draw once it is freed");
    assert_eq!(s.units[u].guys[0].anim, IDLE3);
}

/// The idle roll's thresholds, and `init_real`'s.
#[test]
fn the_variants() {
    assert_eq!(idle_variant(69, false, false), DEFAULT);
    assert_eq!(idle_variant(70, false, false), IDLE1);
    assert_eq!(idle_variant(82, false, false), IDLE1);
    assert_eq!(idle_variant(83, false, false), IDLE2);
    assert_eq!(idle_variant(95, false, false), IDLE2);
    assert_eq!(idle_variant(96, false, false), IDLE3);
    assert_eq!(idle_variant(96, true, false), IDLE1);
    assert_eq!(idle_variant(96, false, true), IDLE1);
    assert_eq!(init_variant(69), DEFAULT);
    assert_eq!(init_variant(79), IDLE1);
    assert_eq!(init_variant(89), IDLE2);
    assert_eq!(init_variant(99), IDLE3);
}

/// **`Animal::do_idle`'s own collision test.** A wander destination
/// that is occupied is not ordered at all — `is_valid`, then
/// `detect_unit_collision(dest, quick 1, …)`. The four draws are spent
/// either way; only the order differs. This is what keeps a herd of
/// sheep standing shoulder to shoulder still, and without it run14's
/// `8/1` walks off on frame 108 where the original's has not moved in
/// 120 frames.
#[test]
fn a_wander_onto_an_occupied_cell_is_not_ordered() {
    use crate::world::{Cell, Terrain};
    // Both sims are the same game to the draw; only the second has a
    // unit standing where the wander wants to go.
    let build = |seed: u32, blocker: Option<Pos>| -> (Sim, usize, u32) {
        let mut world = World::new(40, 40);
        world.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(39, 39));
        let mut s = Sim::new(Tuning::RON, world, 2);
        s.rng = Rng::new(seed);
        let ty = s.add_unit_type(crate::UnitType {
            hits: 10,
            moves: 11,
            combat: crate::combat::Profile {
                block_radius: 48,
                big_radius: 48,
                uber_size: 1,
                ..crate::combat::Profile::default()
            },
            ..crate::UnitType::default()
        });
        s.gaia.herds.push(crate::gaia::Herd {
            cx: 10,
            cy: 10,
            wx: 10,
            wy: 10,
            kind: 408,
            alive: true,
        });
        for slot in 0..4 {
            s.art.lengths.insert((60063, slot), 90);
        }
        let at = herd_centre(10, 10, 10, 10);
        let mut u = Unit::new(8, 1, at, 10);
        u.ty = Some(ty);
        u.herd = Some(0);
        u.guys = vec![Guy {
            cur_time: 89,
            end_time: 90,
            last_time: -1,
            anim: DEFAULT,
            gpiece: 60063,
            flag_piece: 60063,
            stopped: true,
            pending_attack: 0,
            queued_attack: 0,
            aim: None,
            follow: None,
            turret: Turret::ZERO,
        }];
        let a = s.add_unit(u);
        if let Some(p) = blocker {
            let mut b = Unit::new(8, 2, p, 10);
            b.ty = Some(ty);
            s.add_unit(b);
        }
        let before = s.rng.seed;
        s.animal_idle(a);
        (s, a, before)
    };

    // The first word that carries the roll through — three in ten,
    // and then a destination the world accepts.
    let (seed, open, a, before) = (1u32..200)
        .find_map(|n| {
            let (s, a, before) = build(n, None);
            (!s.units[a].orders.is_empty()).then_some((n, s, a, before))
        })
        .expect("a seed whose wander lands");
    let dest = open.units[a]
        .orders
        .front()
        .and_then(crate::orders::Order::move_dest)
        .expect("the open ground gets a wander order");
    let spent = open.rng.seed;
    assert_ne!(spent, before, "the wander's own draws");

    let (blocked, b, _) = build(seed, Some(dest));
    assert!(
        blocked.units[b].orders.is_empty(),
        "the same wander onto an occupied cell is refused"
    );
    assert_eq!(
        blocked.rng.seed, spent,
        "and the refusal costs no draw — the gate is after all four"
    );
}

/// Run12's frame 0: the herd centre of herd 0 at home `(22, 34)`,
/// wander `(22, 34)`, is the cell's centre.
#[test]
fn the_herd_centre() {
    assert_eq!(
        herd_centre(22, 34, 22, 34),
        Pos::new(22 * 768 + 384, 34 * 768 + 384)
    );
}

/// A university and a chain of scholars in it, entry order.
/// `garrison[0]` is the head — the one `ObjectData::get_inside`'s
/// host names in its own `inside_down` — and everything under it is
/// a student.
fn university(anims: &[i8]) -> (Sim, usize, Vec<usize>) {
    let mut s = Sim::new(Tuning::RON, World::new(16, 16), 2);
    let scholar = s.add_unit_type(UnitType {
        hits: 40,
        worker: crate::orders::Worker::Scholar,
        ..UnitType::default()
    });
    let uni = s.add_building(0, Pos::new(3000, 3000), 1);
    let mut us = Vec::new();
    for (i, &anim) in anims.iter().enumerate() {
        let o = i16::try_from(i).unwrap() + 1;
        let mut u = Unit::new(0, o, Pos::new(3000, 3000), 40);
        u.ty = Some(scholar);
        u.on_map = false;
        u.inside = Some(uni);
        u.guys = vec![Guy {
            cur_time: 0,
            end_time: 30,
            last_time: -1,
            anim,
            gpiece: -1,
            flag_piece: -1,
            stopped: true,
            pending_attack: 0,
            queued_attack: 0,
            aim: None,
            follow: None,
            turret: Turret::ZERO,
        }];
        let u = s.add_unit(u);
        s.buildings[uni].garrison.push(u);
        us.push(u);
    }
    (s, uni, us)
}

/// **The `0x20` tie-break counts the guy asking** — `Guy::set_anim
/// @005da300:332`, and `docs/ANIM.md` §4.12.
///
/// The walk starts at the host's own `inside_down` and steps down the
/// chain to its tail, so every member is examined and the requester is
/// one of them; its `guys[0].cur_anim` is still the slot it is
/// *leaving*, because `set_anim` writes the new one a hundred lines
/// later. Two consequences, and both are asserted here:
///
/// - a student **already on `0x20`** that re-rolls variant 3 can never
///   land on `0x20` again — it takes `0x1d` every time, which is the
///   whole of Great Lakes 10274 and so of the word at 10303;
/// - a student on any **other** slot still takes `0x20`, unless some
///   *other* member of its chain is on it. The self-clause widens the
///   rule; it does not replace it.
///
/// The three variants under 3 never reach the walk at all, and the
/// head takes the `+0x19` set whatever anyone else is playing.
///
/// **Made to fail on purpose** with `c != u` back in the predicate:
/// the first assertion reads 0x20 where the dump reads 0x1d, which is
/// run100's `1/51` on 10274 exactly.
#[test]
fn the_scholar_s_0x20_tie_break_counts_the_guy_asking() {
    // Head on a teach slot, one student already on 0x20 — run100's
    // chain A, `1/44` over `1/51`.
    let (s, _, us) = university(&[0x19, 0x20]);
    assert_eq!(
        s.scholar_slot(us[1], 3),
        0x1d,
        "a student re-rolling variant 3 off 0x20 must bounce to 0x1d"
    );
    // The other three variants are not the walk's business.
    for v in 0..3 {
        assert_eq!(s.scholar_slot(us[1], v), v + 0x1d, "variant {v}");
    }
    // And the head is the `+0x19` set whatever the chain is playing.
    assert_eq!(s.scholar_slot(us[0], 3), 3 + 0x19);

    // A student on another slot, alone in taking it: 0x20 stands.
    let (s, _, us) = university(&[0x19, 0x1d]);
    assert_eq!(
        s.scholar_slot(us[1], 3),
        0x20,
        "the tie-break fired on a chain with nobody on 0x20"
    );

    // …and a *third* member already on it takes it away again, which
    // is the reading this crate had and it was never wrong, only
    // narrow.
    let (s, _, us) = university(&[0x19, 0x1d, 0x20]);
    assert_eq!(s.scholar_slot(us[1], 3), 0x1d);
}

/// **§15: a unit that goes inside in its own work still takes its
/// figures' frame.** `Unit::process@00610bc0` runs the work (`+0x188`)
/// and then `Guy::process` on every figure under the entry's
/// `inside_up < 0` test alone, so a walker that casts its boat at the
/// shore (`crate::Sim::cast_transport`) and is cargo by the work's end
/// still has its figures moved once: a crew figure standing on its
/// `des` writes `last_speed` 0, decays its average and turns to
/// `des_angle`, paying the turn's `set_anim`. East Indies' `1/77` on
/// 19413 is the case (run269), with the angles here.
#[test]
fn a_walker_that_boards_its_boat_moves_its_figures_that_frame() {
    use crate::movement::{Angle, Body};
    use crate::world::{Cell, Terrain, UNITS_PER_TILE, tile};
    let tile_pos = |tx: i32, ty: i32| {
        Pos::new(
            tx * UNITS_PER_TILE + UNITS_PER_TILE / 2,
            ty * UNITS_PER_TILE + UNITS_PER_TILE / 2,
        )
    };
    // `transport.rs`'s shore: land on the left eight cells, ocean on
    // the right four, the shore column flagged with the sea beside it.
    let mut w = World::new(12, 8);
    w.fill_region(Terrain::Land, Cell::new(0, 0), Cell::new(7, 7));
    let sea = w.fill_region(Terrain::Sea, Cell::new(8, 0), Cell::new(11, 7));
    for y in 0..8 {
        for x in 0..12 {
            let c = Cell::new(x, y);
            let mut d = w.cell_data(c);
            if x >= 8 {
                d.land = 2;
                for tx in x * 4..x * 4 + 4 {
                    for ty in y * 4..y * 4 + 4 {
                        w.set_tile_field(Pos::new(tx, ty), tile::SURFACE, tile::SURFACE_OCEAN);
                    }
                }
            } else if x == 7 {
                d.flags |= 0x100;
                d.region2 = Some(sea);
            }
            w.set_cell_data(c, d);
        }
    }
    let mut s = Sim::new(Tuning::RON, w, 2);
    s.nation[1].human = false;
    let walker = s.add_unit_type(UnitType {
        hits: 40,
        ..UnitType::default()
    });
    let mut barge = UnitType {
        hits: 50,
        moves: 25,
        ..UnitType::default()
    };
    barge.combat.domain = crate::attrition::Domain::Sea;
    barge.combat.block_radius = 48;
    let b = s.add_unit_type(barge);
    s.unit_types[b].tree = Some(crate::transport::ty::TRANSPORTBARGE);
    s.unit_types[b].type_index = crate::transport::ty::TRANSPORTBARGE as i32;
    let index = s.find_free(1, crate::UNIT_BASE, crate::BUILD_BASE).unwrap();
    let mut unit = Unit::new(1, index, tile_pos(30, 14), 40);
    unit.ty = Some(walker);
    let u = s.add_unit(unit);
    s.init_guys(u, Some(walker));
    // The second figure: standing on its `des`, a step's speed still
    // on it, owed the turn run269's `1/77` took on 19413.
    let at = Pos::new(tile_pos(30, 14).x - 48, tile_pos(30, 14).y - 192);
    let mut crew = s.units[u].guys[0];
    crew.follow = Some(Follow {
        body: Body {
            pos: at,
            last_speed: 1,
            avg_speed: 16,
        },
        des: at,
        facing: Angle(-541_917_184),
        des_angle: Angle(-901_447_680),
        track: (-48, -192),
    });
    s.units[u].guys.push(crew);
    s.units[u].auto_transport = true;
    s.add_move_order(
        u,
        tile_pos(40, 14),
        crate::orders::MoveKind::MoveTo,
        crate::orders::QueuePos::New,
        false,
    );
    s.units[u].path.push(crate::orders::PathData {
        to: tile_pos(33, 14),
        tolerance: 0,
        flags: crate::orders::path_flag::TRANSPORT,
    });
    assert!(!s.set_new_location(u, tile_pos(33, 14), false));

    s.trace_phases = true;
    let mut events = Vec::new();
    s.process_unit(u, 1, &mut events);
    assert!(s.units[u].inside_unit.is_some(), "the walker is cargo");
    let f = s.units[u].guys[1].follow.expect("the crew figure");
    assert_eq!(f.facing, f.des_angle, "it turned, as the original's did");
    assert_eq!(f.body.last_speed, 0, "`Guy::move`'s standing arm");
    assert_eq!(f.body.avg_speed, 12, "(16 × 3 + 0) / 4");
    assert!(
        s.phase_marks.iter().any(|(l, _)| l == SITE_TURN_STAND),
        "the turn's draw"
    );
}
