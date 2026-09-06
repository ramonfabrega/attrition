//! The `UNIT` record: position, figures, collision, angle, sight, packed.

use super::*;

/// One unit whose position the two sides disagree on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Divergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    pub ours: Pos,
    pub theirs: Pos,
}

/// Which of a unit's two angles disagreed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    /// `UnitData::angle` (`+0x50`) against `Movement::heading` — the bearing
    /// to the destination, which `Unit::set_angle` writes outright.
    Heading,
    /// Guy 0's `angle` (`+0x18`) against `Movement::facing` — the direction
    /// the step was actually taken along, which only the turn rate moves.
    Facing,
}

/// One unit whose heading or facing the two sides disagree on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AngleDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    pub which: Which,
    pub ours: i32,
    pub theirs: i64,
}

/// One field of a unit's collision block the two sides disagree on —
/// `docs/COLLISION.md` §8. The block is written at every detail level, so
/// this is compared on every capture the harness reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollideDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    /// The field, named as `UnitData::log_data` writes it.
    pub field: &'static str,
    pub ours: i64,
    pub theirs: i64,
}

/// One unit whose line of sight the two sides disagree on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LosDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    pub ours: i32,
    pub theirs: i64,
}

/// One unit-frame whose `unit_masks & 0x80000` disagreed — the packed bit
/// `Unit::init` sets, `SpellType::cast_pack` sets again and
/// `SpellType::cast_unpack` clears (`docs/ORDERS.md` §6.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackedDivergence {
    pub frame: i64,
    pub who: i64,
    pub o: i64,
    pub ours: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diff::testkit::*;

    use crate::testenv::{dump, install};

    /// **The guy stack's *length*, against every dump that names one.**
    ///
    /// `Unit::init@00612100:471`-`508` sizes a unit's figures to
    /// `crew_size + squad_size` and gives every slot its own
    /// `Guy::init_real`. `squad_size` is not a column: `UnitType::init@
    /// 0061ab50:723` writes the literal 1 into every type before it reads
    /// `UBER_SIZE` and `CREW_SIZE` beside it. So the count is
    /// [`sim::anim::SQUAD_SIZE`] plus `unitrules.xml`'s `CREW_SIZE` — one
    /// for a Citizen, two for a Scout (the dog), three for a Caravan, four
    /// for a Trebuchet.
    ///
    /// The count was a hardcoded 1 in [`sim::Sim::init_guys`] until item
    /// 173, which cost East Indies' long word frame **6164**: the AI's
    /// first Caravan spent one `Guy::init_real+0x52` where the original
    /// spent three, and the two figures' first idles were the two
    /// `Guy::set_anim+0x97a < Unit::do_idle+0x7d` missing from 6165. A
    /// unit stood up *from* a dump never had the bug — `build_sim` gives
    /// it the `GUY` blocks the file prints — which is why nothing caught
    /// it for a month and why this assertion is worth having: it checks
    /// the **install's** answer against the dump's on every unit of every
    /// capture, born or stood up.
    #[test]
    fn every_dumped_unit_has_crew_size_plus_one_figures() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let mut rows = 0usize;
        let mut counts: std::collections::BTreeMap<usize, usize> =
            std::collections::BTreeMap::new();
        for name in [
            "gamelog-run12-dumpall-seeds.txt",
            "gamelog-run13-window-95-105.txt",
            "gamelog-run20-islands-dumpall.txt",
            "gamelog-run3-fulldump-types.txt",
            "gamelog-run22-islands-dock-window.txt",
            "gamelog-run25-islands-emergency-window.txt",
            "gamelog-run44-islands-turners.txt",
            "gamelog-run27-islands-defending-window.txt",
            "gamelog-run34-greatlakes-dumpall-start.txt",
            "gamelog-run58-islands-5k2.txt",
        ] {
            let Some(path) = dump(name) else { continue };
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let Some(init) = log.initial() else { continue };
            let built = build_sim(&loaded, &init, Tuning::RON);
            for u in &init.units {
                // A `GUY` block is what states the count; a dump below
                // `DUMP_ALL` prints none and its units arrive empty.
                if u.guys.is_empty() {
                    continue;
                }
                // Gaia's animals are not `Unit::init`'s: `Gaia::spawn_*`
                // builds them, and their crew is the herd's own business
                // (`docs/ANIM.md` section 7).
                if !(0..8).contains(&u.who) {
                    continue;
                }
                let Some(link) = built.units.iter().find(|l| l.who == u.who && l.o == u.o) else {
                    continue;
                };
                let Some(ty) = link.kind else { continue };
                let crew = built.sim.unit_types[ty].combat.crew_size;
                let ours = usize::try_from(crew + sim::anim::SQUAD_SIZE as i32).unwrap_or(0);
                rows += 1;
                *counts.entry(u.guys.len()).or_default() += 1;
                assert_eq!(
                    ours,
                    u.guys.len(),
                    "{name}: unit {}/{} (type {:?}) has {} figures in the \
                     dump; CREW_SIZE {crew} + SQUAD_SIZE says {ours}",
                    u.who,
                    u.o,
                    u.guys.first().and_then(|g| g.kind),
                    u.guys.len(),
                );
            }
        }
        eprintln!("guy-count walk: {rows} units, lengths {counts:?}");
        // The floor is the walk's own: a run with no multi-figure unit in
        // it would pass on ones alone and say nothing.
        assert!(
            rows >= 60 && counts.keys().any(|&n| n > 1),
            "the units those dumps name between them: {rows} over {counts:?} \
             — the floor is 60, at least one of them crewed"
        );
    }

    /// **The frozen clock: `unit_masks2 & 0x10` is the step, not a
    /// modifier on it** — the assertion the ANIM docs-versus-code pass
    /// asked for (2026-09-05).
    ///
    /// `docs/ANIM.md` §5 states the step as `1` (`2` under `guy_flags & 4`
    /// with an `ATTACK2` playing; **`0` while `unit_masks2 & 0x10`**) and
    /// `sim::Sim::guy_inc_time` steps by one always, with no mention of
    /// the zero arm. The bit is `Unit::fight@005fd4d0`'s, set on the arm
    /// where a unit is swinging and cleared by `Unit::process@00610bc0`,
    /// so it is a one-frame freeze on a unit in melee — and nothing in
    /// this workspace has ever read it.
    ///
    /// Nothing needed to be *inferred* to check it, which is the point.
    /// `GuyData::log_data` prints `last_time` beside `cur_time`, and
    /// `last_time` is `cur_time` **before this frame's step**; their
    /// difference is therefore the step the original actually took, per
    /// figure, per frame, already on disk. A `set_anim` writes `last_time`
    /// −1 and a wrap takes `cur_time` backwards, so those two are skipped
    /// and everything else is the arithmetic.
    ///
    /// The corpus is not short of it: `gamelog-run17-combat.txt` prints 35
    /// unit-frames with the bit set and `gamelog-run44-islands-turners.txt`
    /// 26, of which run44's carry the whole `GUY` block. This walks every
    /// capture that prints both.
    ///
    /// It is the **original's** rule that is asserted here, not the
    /// simulation's behaviour: `crate::diff` compares no clock on a frame
    /// a unit is in melee, so a diff cannot yet fail on the gap. What this
    /// does is make the gap falsifiable and keep the rule from drifting —
    /// the day `guy_inc_time` learns the bit, this is the check that says
    /// what it should do.
    #[test]
    fn the_frozen_frame_s_figures_do_not_step_their_clocks() {
        /// `UnitData::unit_masks2` bit `0x10` — `Unit::fight`'s swing mark.
        const FROZEN: i64 = 0x10;
        let mut frozen_seen = 0usize;
        let mut frozen_steps: std::collections::BTreeMap<i64, usize> =
            std::collections::BTreeMap::new();
        let mut free_steps: std::collections::BTreeMap<i64, usize> =
            std::collections::BTreeMap::new();
        let mut names = Vec::new();
        for name in [
            "gamelog-run44-islands-turners.txt",
            "gamelog-run25-islands-emergency-window.txt",
            "gamelog-run27-islands-defending-window.txt",
            "gamelog-run20-islands-dumpall.txt",
            "gamelog-run13-window-95-105.txt",
        ] {
            let Some(path) = dump(name) else { continue };
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            names.push(name);
            for frame in log.frame_states() {
                for u in &frame.units {
                    let Some(m2) = u.unit_masks2 else { continue };
                    let frozen = m2 & FROZEN != 0;
                    for g in &u.guys {
                        let (Some(cur), Some(last)) = (g.cur_time, g.last_time) else {
                            continue;
                        };
                        // `set_anim` writes −1; a wrap takes the clock back.
                        if last < 0 || cur < last {
                            continue;
                        }
                        let step = cur - last;
                        if frozen {
                            frozen_seen += 1;
                            *frozen_steps.entry(step).or_default() += 1;
                        } else {
                            *free_steps.entry(step).or_default() += 1;
                        }
                    }
                }
            }
        }
        eprintln!(
            "the frozen frame over {} capture(s) {names:?}: {frozen_seen} figure-frames \
             with `unit_masks2 & 0x10`, steps {frozen_steps:?}; without it {free_steps:?}",
            names.len()
        );
        if names.is_empty() {
            eprintln!("skipping: no capture (set RON_GAMELOG_DIR)");
            return;
        }
        // The floor: without a frozen figure-frame on disk the walk proves
        // nothing and would pass on an empty set, which is the failure a
        // corpus test is least able to report about itself.
        assert!(
            frozen_seen > 0,
            "no capture in the walk carries a figure-frame with `unit_masks2 & 0x10`; \
             the check is vacuous. `gamelog-run44-islands-turners.txt` had them"
        );
        assert_eq!(
            frozen_steps.keys().copied().collect::<Vec<_>>(),
            vec![0],
            "a figure whose unit carries `unit_masks2 & 0x10` stepped its clock: \
             {frozen_steps:?}"
        );
        // And the other side of it, so the assertion above cannot be
        // satisfied by a corpus in which nothing steps at all.
        assert!(
            free_steps.contains_key(&1),
            "no figure stepped by one anywhere in the walk: {free_steps:?}"
        );
    }

    /// **`get_unit_gpiece`'s walk, against every piece a dump names.**
    ///
    /// [`sim::Sim::unit_gpiece`] derives a guy's graphic piece from four
    /// coordinates — the nation's `UNIT_CONTINENT`, the leader's age
    /// bracket, the gender bit (or the packed bit, which sits in the same
    /// slot) and the crew index — and then walks *down* from it until the
    /// install's own `<UNIT>` entries have one. Every dump that carries a
    /// `GUY` block prints the answer the original reached, per guy, so the
    /// walk is checkable to the **number** rather than to the arithmetic:
    /// this asserts it over every one of them.
    ///
    /// It is what makes the derivation worth having. [`sim::anim::Art`]'s
    /// `pieces` is seeded from the opening dump alone, so a type absent
    /// from the opening — a Fisherman the AI trains on frame 4,376 — had
    /// **no** piece at all, an `end_time` of [`sim::anim::UNKNOWN`], and no
    /// animation of its ever wrapped (item 152).
    #[test]
    fn the_walk_gives_every_dumped_guy_its_own_piece() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        let mut rows = 0usize;
        let mut pieces: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
        for name in [
            "gamelog-run12-dumpall-seeds.txt",
            "gamelog-run13-window-95-105.txt",
            "gamelog-run20-islands-dumpall.txt",
            "gamelog-run3-fulldump-types.txt",
            "gamelog-run22-islands-dock-window.txt",
            "gamelog-run25-islands-emergency-window.txt",
            "gamelog-run44-islands-turners.txt",
            "gamelog-run27-islands-defending-window.txt",
            "gamelog-run34-greatlakes-dumpall-start.txt",
            "gamelog-run58-islands-5k2.txt",
        ] {
            let Some(path) = dump(name) else { continue };
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let Some(init) = log.initial() else { continue };
            let built = build_sim(&loaded, &init, Tuning::RON);
            for u in &init.units {
                // Gaia's pieces come off `first_bird_piece`, a runtime
                // pointer no file states — the dump stays their source.
                if !(0..8).contains(&u.who) {
                    continue;
                }
                let Some(link) = built.units.iter().find(|l| l.who == u.who && l.o == u.o) else {
                    continue;
                };
                let Some(ty) = link.kind else { continue };
                let packed = built.sim.units[link.unit].combat.packed;
                for (n, g) in u.guys.iter().enumerate() {
                    let Some(theirs) = g.gpiece else { continue };
                    rows += 1;
                    pieces.insert(theirs);
                    assert_eq!(
                        built
                            .sim
                            .unit_gpiece(u.who as u8, ty, u.o as i16, n as u8, packed),
                        Some(theirs as i32),
                        "{name}: unit {}/{} guy {n} (type {:?}, packed {packed})",
                        u.who,
                        u.o,
                        g.kind
                    );
                }
            }
        }
        eprintln!(
            "gpiece walk: {rows} guys over {} distinct pieces",
            pieces.len()
        );
        assert!(
            rows >= 60,
            "the guys those dumps name between them: {rows}, the floor is 60"
        );
    }

    /// **The player units' lengths, from the install against the dumps.**
    ///
    /// The gaia check above says the `.bha` arithmetic is right; this one
    /// says the *addressing* is — that
    /// `GraphicPieces::init_piece_ranges@008f70e0`'s strides and
    /// `get_unit_gpiece@0090c030`'s sum put each `<UNIT name="…">` entry at
    /// the piece number the original hands out. Eight dumps between them
    /// print 187 `(gpiece, cur_anim) → end_time` rows over six pieces of
    /// two nations, and every one of them has to be the install's own.
    ///
    /// **The list is the assertion, and it was five dumps too short.**
    /// Five starts and short windows print only the idles, the walks, the
    /// chop, the sow and the reap; the wood dump — `CHAR_DUMP_WOOD`,
    /// **32** frames on every citizen piece the corpus names — was in no
    /// dump this test read, and the install's table said 33 for a month
    /// (item 121, `docs/ANIM.md` §3.1). Item 87's ledger, one row of it
    /// paid: a slot the parser had and nothing compared.
    ///
    /// **Except a mirrored guy's, and that is the check's other half.** A
    /// crew member past the squad's size copies guy 0's `cur_anim` and
    /// `cur_time` every frame and keeps its **own** `end_time`
    /// (`docs/ANIM.md` §5), so the pair a dump prints for the scouts' dogs
    /// is not a length row at all: it is one animation's slot beside
    /// another's length. Five of the 88 are that, all on the two dogs
    /// (crew 1, `13043` and `12691`), and each is a length the *same piece*
    /// carries at another slot — which is what says the rows are the
    /// mirror rather than a mis-addressed piece.
    #[test]
    fn the_install_s_piece_lengths_match_the_dumps() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        assert_eq!(
            loaded.piece_lengths.len(),
            1440,
            "every `<UNIT>` entry the unit path can name"
        );
        // **And an entry with no `<ANIM>` child is one of them.** All
        // sixty animation-less `<UNIT>`s in the shipped file are
        // `-CREW{k}`, and they are eighty-one pieces here because a
        // `GRAPH` is shared. The piece exists — it loads a model — and
        // its packet is *empty*, so every slot is the three frames
        // `AnimationPacket::get_game_frames` returns and none of them
        // loops: that is what makes a caravan's two crew figures fall to
        // the idle and roll every third frame (`docs/ANIM.md` §3.6).
        // Until 2026-09-02 the reader dropped the entry, `get_unit_gpiece`
        // fell to `first_unit_piece` and the crew walked on the citizen's
        // art.
        for p in [12681, 25353] {
            assert_eq!(
                loaded
                    .piece_lengths
                    .get(&p)
                    .map(std::collections::BTreeMap::len),
                Some(0),
                "the caravan's crew piece {p} is an entry with an empty packet"
            );
        }
        // The pieces run12's guys name, by the arithmetic
        // (`artdata::tests::a_unit_graphic_s_name_gives_its_piece`): player
        // 0 is Nubian (`UNIT_CONTINENT 1 Arab`, one style stride of
        // `0x160`) and player 1 British (`0 European`, style 0), the scout
        // is `TypeIndex` 69 and the citizen 50, and a dog is crew 1
        // (`+0x3180`). Each of the six is a `<UNIT>` entry the install
        // names, and a mis-addressed table would miss one.
        for p in [371, 13043, 19, 12691, 352, 6688] {
            assert!(
                loaded.piece_lengths.contains_key(&p),
                "no install entry for piece {p}"
            );
        }
        // The scout's idle variants, which are the whole of item 71: its
        // `CHAR_DEFAULT` is 61 frames and its `CHAR_IDLE1` **76**, and no
        // dump has ever shown the second — so a roll that took `IDLE1`
        // used to be played as the default and the clock wrapped fifteen
        // frames early.
        let scout = &loaded.piece_lengths[&371];
        assert_eq!(scout.get(&sim::anim::DEFAULT), Some(&61));
        assert_eq!(scout.get(&sim::anim::IDLE1), Some(&76));
        assert_eq!(scout.get(&sim::anim::IDLE2), Some(&41));
        assert_eq!(scout.get(&sim::anim::IDLE3), Some(&190));
        // And no unit piece in the shipped file has a `CHAR_GROUP_IDLE2` —
        // so `set_anim`'s captain gate, the one frame in sixteen that
        // skips the idle roll, can never fire (§4.2). It was `Art::
        // group_idle`, empty for want of a dump; it is now a fact.
        assert!(
            loaded
                .piece_lengths
                .values()
                .all(|m| !m.contains_key(&sim::anim::GROUP_IDLE2)),
            "no unit packet names a group idle"
        );
        // **And no piece any traced unit carries has a turn animation** —
        // which is what makes `Guy::do_turn@005d97a0:15` unreachable on
        // every capture there is. That arm overrides `Guy::move`'s
        // standing walk (`crates/sim/src/anim.rs`, `guys_follow`) with
        // `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` when `guy_flags & 8`, and
        // `Guy::init_real@005db6b0:179` sets that bit only when the guy's
        // piece names one. 273 of the install's 1,359 unit pieces do, so
        // the mechanic is real; **none of the eight a `DUMP_ALL` run's
        // guys name is among them** — the two scouts, their two dogs and
        // the six citizens of a Nubian and a British start — and gaia's
        // six pieces are not `<UNIT>` entries at all, so they cannot carry
        // the bit either. Booked as item 36's second half; closed here,
        // from the install, rather than modelled (`docs/ANIM.md` §4.6).
        for p in [0, 19, 352, 371, 6336, 6688, 12691, 13043] {
            let m = &loaded.piece_lengths[&p];
            assert!(
                !m.contains_key(&sim::anim::TURN_LEFT) && !m.contains_key(&sim::anim::TURN_RIGHT),
                "piece {p} names a turn animation, so `guy_flags & 8` is live"
            );
        }
        assert_eq!(
            loaded
                .piece_lengths
                .values()
                .filter(|m| m.contains_key(&sim::anim::TURN_LEFT))
                .count(),
            273,
            "the pieces that do have one — the check is that it is neither 0 nor all"
        );

        let mut rows = 0usize;
        let mut slots: std::collections::BTreeSet<i8> = std::collections::BTreeSet::new();
        let mut mirrored = Vec::new();
        for name in [
            "gamelog-run12-dumpall-seeds.txt",
            "gamelog-run13-window-95-105.txt",
            "gamelog-run20-islands-dumpall.txt",
            "gamelog-run3-fulldump-types.txt",
            "gamelog-run22-islands-dock-window.txt",
            // The three the list wanted and did not have (2026-08-31,
            // item 121). The five above are starts and short windows, so
            // between them they print only the idles, the walks, the chop,
            // the sow and the reap — and the eight slots they never reach
            // are where the install's table was wrong. run25 brings the
            // three attacks, the two dumps and the ore half; run44 the
            // turns and pack/unpack; run27 `CHAR_WALK_WITH_ORE`.
            "gamelog-run25-islands-emergency-window.txt",
            "gamelog-run44-islands-turners.txt",
            "gamelog-run27-islands-defending-window.txt",
        ] {
            let Some(path) = dump(name) else { continue };
            let text = std::fs::read_to_string(&path).unwrap();
            let log = Log::parse(&text);
            let Some(init) = log.initial() else { continue };
            let built = build_sim(&loaded, &init, Tuning::RON);
            let sim = &built.sim;
            for (&(who, _, _, guy), &piece) in &sim.art.pieces {
                // Gaia is the other check's.
                if who >= 8 {
                    continue;
                }
                let theirs = loaded
                    .piece_lengths
                    .get(&piece)
                    .unwrap_or_else(|| panic!("{name}: no install entry for piece {piece}"));
                for (&(p, slot), &n) in &sim.art.lengths {
                    if p != piece {
                        continue;
                    }
                    rows += 1;
                    slots.insert(slot);
                    if theirs.get(&slot) == Some(&n) {
                        continue;
                    }
                    // A mirrored guy: the slot is guy 0's and the length is
                    // this one's, so it must be *some* slot of this piece.
                    assert!(guy > 0, "{name}: piece {piece} slot {slot} says {n}");
                    assert!(
                        theirs.values().any(|&m| m == n),
                        "{name}: piece {piece} slot {slot} says {n}, which is no \
                         slot of its own"
                    );
                    mirrored.push((piece, slot, n));
                }
            }
        }
        eprintln!(
            "piece lengths: {rows} rows over slots {:?}",
            slots.iter().collect::<Vec<_>>()
        );
        assert!(
            rows >= 88,
            "the rows eight dumps between them print: {rows}, the floor is 88"
        );
        mirrored.sort_unstable();
        mirrored.dedup();
        assert_eq!(
            mirrored,
            vec![
                (12691, sim::anim::DEFAULT, 76),
                (12691, sim::anim::IDLE1, 190),
                (12691, sim::anim::IDLE3, 41),
                (13043, sim::anim::IDLE1, 190),
                (13043, sim::anim::IDLE2, 61),
                (13043, sim::anim::IDLE2, 190),
            ],
            "the mirrored rows, all on the two dogs"
        );
    }

    /// **run56's figures, frame for frame** — every `GUY` block's own
    /// `x`, `y` and `angle`, over East Indies' 3,000-frame capture.
    ///
    /// This is the record behind item 128. A unit is a `Unit` and one or
    /// more `Guy`s; guy 0's body is the unit's own, and a **crew** guy
    /// whose piece names a track offset walks a second body toward a
    /// destination of its own. A scout is the pair that shows it: the man
    /// arrives and the dog is still walking four frames later, which is
    /// four frames on which the two figures answer `Guy::set_anim`'s
    /// walking-guy early return differently (`docs/MOVEMENT.md`, "The
    /// follower's destination").
    ///
    /// Nothing here is installed. The bodies are **derived** — the crew's
    /// from `Sim::seat_guys` at the start and from guy 0's own
    /// `Guy::set_new_location` and `Guy::set_angle` after that — so every
    /// row is a prediction the dump can refuse. The whole record is
    /// compared, both figures of every unit gaia's included, because nine
    /// tenths of a dumped record once went uncompared for a month.
    #[test]
    fn run56_s_figures_stand_where_the_original_s_do() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        let theirs = std::mem::take(&mut init.frame_bodies);
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);
        let last = theirs.last().map_or(0, |(n, _)| *n);
        assert!(
            last >= 2_990,
            "run56's body table reaches frame {last}, wanted 2,990+ — this \
             is the long East Indies capture at full detail, and a short \
             file here is a wrong file"
        );
        let mut built = build_sim(&loaded, &init, Tuning::RON);
        let mut compared = 0usize;
        // Three tallies, because the rows fall into three kinds and only
        // one of them is this mechanic's.
        let mut gaia_bearings = 0usize;
        let mut on_the_quit = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..=last {
            built.tick();
            let Some((_, units)) = theirs.iter().find(|(n, _)| *n == f) else {
                continue;
            };
            for state in units {
                let Some(u) = built
                    .units
                    .iter()
                    .find(|l| l.who == state.who && l.o == state.o)
                    .map(|l| l.unit)
                else {
                    continue;
                };
                let unit = &built.sim.units[u];
                for (n, g) in state.guys.iter().enumerate() {
                    let (Some(pos), Some(angle)) = (g.pos, g.angle) else {
                        continue;
                    };
                    let (ours, facing) = match unit.guys.get(n).and_then(|x| x.follow) {
                        Some(b) => (b.body.pos, b.facing),
                        None => (unit.movement.body.pos, unit.movement.facing),
                    };
                    compared += 3;
                    let placed = ours.x == pos.x as i32 && ours.y == pos.y as i32;
                    let aimed = i64::from(facing.0) == angle;
                    if placed && aimed {
                        continue;
                    }
                    // **Gaia's bearing is not modelled**, and it is not the
                    // body: `Sim::reseat_animal` puts an animal back on the
                    // original's own position every traced frame, so the
                    // place agrees and the angle never does. Counted, not
                    // asserted away.
                    if state.who >= 8 && placed {
                        gaia_bearings += 1;
                        continue;
                    }
                    // The quit's own frame is the capture's last and is
                    // half a frame: the dump is written before the rest of
                    // it runs. Every widening on this capture carries the
                    // same exemption.
                    if f == last {
                        on_the_quit += 1;
                        continue;
                    }
                    if wrong.len() < 8 {
                        wrong.push(format!(
                            "frame {f}: {}/{} guy {n} ours ({}, {}) angle {} \
                             theirs ({}, {}) angle {}",
                            state.who, state.o, ours.x, ours.y, facing.0, pos.x, pos.y, angle
                        ));
                    }
                }
            }
        }
        eprintln!(
            "run56 bodies: {compared} fields compared, {} rows wrong \
             ({gaia_bearings} gaia bearings, {on_the_quit} on the quit's frame)",
            wrong.len()
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 1_060_000,
            "the record is being read: {compared} fields"
        );
        // **Every figure of every player's unit stands where the
        // original's does, on all 3,000 frames** — guy 0 and the crew
        // alike, and the crew's position is derived rather than read.
        assert!(
            wrong.is_empty(),
            "a figure stands somewhere the original's does not: {wrong:?}"
        );
        // Asserted as they stand, so that closing either moves the number
        // rather than passing quietly. The one on the quit's frame is
        // player 0's Citizen `o 5`, eighteen units short on both axes.
        assert_eq!(gaia_bearings, 301_810, "gaia's unmodelled bearings");
        assert_eq!(
            on_the_quit, 1,
            "rows on the capture's half-written last frame"
        );
    }

    /// **run56's collision block, over three thousand frames** — the
    /// same five fields run10 pins on Great Lakes, asked of East Indies'
    /// longest full-detail capture.
    ///
    /// The scored East Indies run is run39's 1,850 frames; run56 is the
    /// same game at the same detail carried to 3,000, and until now the
    /// only halves of its `UNITDATA` anything compared were the figures
    /// and the gather record. The collision block is compared on every
    /// frame the harness reads — `UnitData::log_data` writes all five at
    /// every detail level — and nothing asserted it here.
    #[test]
    fn run56_s_collision_block_agrees_past_the_scored_length() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr)) = (
            dump("gamelog-run56-islands-3k.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run56.log"),
        ) else {
            eprintln!("skipping: no run56 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let refs: Vec<&Initial> = vec![&sib_init];
        let report = run_traced(&loaded, &log, Tuning::RON, None, None, &refs, Some(&tr)).unwrap();
        assert!(
            report.frames.len() >= 3_000,
            "run56's length is {} — a short file here is a wrong file",
            report.frames.len()
        );
        let seen: usize = report.frames.iter().map(|f| f.collide_compared).sum();
        let parted: std::collections::BTreeMap<(i64, i64), i64> = report
            .first_divergence_by_unit()
            .into_iter()
            .map(|(w, o, f)| ((w, o), f))
            .collect();
        let bad: Vec<CollideDivergence> = report
            .frames
            .iter()
            .flat_map(|f| f.collide_diverged.iter().copied())
            .filter(|d| parted.get(&(d.who, d.o)).is_none_or(|&f| d.frame < f))
            .collect();
        eprintln!(
            "run56 collision: {seen} field-frames compared, {} wrong",
            bad.len()
        );
        for d in bad.iter().take(16) {
            eprintln!(
                "  frame {} {}/{} {} ours {} theirs {}",
                d.frame, d.who, d.o, d.field, d.ours, d.theirs
            );
        }
        // **249,413 field-frames, and not one of them wrong** — where
        // run10 pins 139,514 on the other map. The block is scoped to
        // unit-frames whose *positions* already agree, so the number is
        // this capture's own size rather than a score, and it moves when
        // a longer East Indies capture replaces this one.
        //
        // It was **249,293** while the AI's Dock stood two cells north of
        // the original's and its builder walked, by luck, almost the
        // original's own line. The dock's slide (`docs/AI.md` §21) put the
        // dock right and the *approach* wrong — 249,288, `1/11` parting a
        // frame earlier — and `find_nearby_spot`'s missing terrain test
        // (`docs/ORDERS.md` §10) put that right too. **`1/11` now agrees
        // for the whole capture**, and the only unit that parts at all is
        // player 0's Citizen `o 5` on the quit's own half-written frame.
        assert_eq!(
            seen, 249_413,
            "five fields on every agreeing unit-frame of run56"
        );
        assert!(
            bad.is_empty(),
            "the collision block agrees on every comparable field-frame of {seen}: {bad:?}"
        );
    }

    /// **The crew's follow offsets, from the install** — `track_dx` and
    /// `track_dy`, which decide whether a unit's second figure stands on
    /// the first or walks its own body behind it
    /// (`docs/MOVEMENT.md`, "The follower's destination").
    ///
    /// The two scout dogs are the pair that matters, because they are the
    /// headline's own divergence: both nations' `-CREW1` entry writes
    /// `trackoffsetx="-20" trackoffsety="10" scale="1"`, and `guy_scale`
    /// is the executable's 4.8, so the stored pair is `(-96, 48)`. A
    /// citizen has one figure and no entry here at all.
    /// **The crew's follow offsets, from the install** — `track_dx` and
    /// `track_dy`, which decide whether a unit's second figure stands on
    /// the first or walks its own body behind it
    /// (`docs/MOVEMENT.md`, "The follower's destination").
    ///
    /// The two scout dogs are the pair that matters, because they are the
    /// headline's own divergence: both nations' `-CREW1` entry writes
    /// `trackoffsetx="-20" trackoffsety="10" scale="1"`, and `guy_scale`
    /// is the executable's 4.8, so the stored pair is `(-96, 48)`. A
    /// citizen has one figure and no entry here at all.
    #[test]
    fn the_install_s_crew_tracks_are_the_scout_dog_s() {
        let Some(inst) = install() else { return };
        let loaded = crate::load::load(&inst).unwrap();
        for dog in [13043, 12691] {
            assert_eq!(
                loaded.piece_tracks.get(&dog).copied(),
                Some((-96, 48)),
                "piece {dog} is a scout's dog and carries the -20/10 offset"
            );
        }
        // A guy-0 piece has no offset: the scouts themselves, and the
        // citizens, which are one figure each.
        for lone in [371, 19, 352, 6688] {
            assert!(
                !loaded.piece_tracks.contains_key(&lone),
                "piece {lone} is a guy 0 and names no track offset"
            );
        }
    }

    /// **run65 — the caravan's turn out of its own city, every unit, every
    /// frame** (2026-09-02).
    ///
    /// run54's game to 6,220 frames with a `DUMP_ALL` window on
    /// `[6196, 6214)` — the eighteen `FRAME` blocks either side of the
    /// word's own parting — at `MISC` alone otherwise.
    /// `rngcmp.py rontrace-run54.log rontrace-run65.log`: **6,221 frames,
    /// zero differing**, so it is run54's game and the fifth capture in a
    /// row for which a window costs the stream nothing.
    ///
    /// **What it was booked for.** East Indies' word parted at 6207 on the
    /// caravan's first leg: the original stood at its home city and walked
    /// on sim-frame 6207 where this crate walked on 6206, one frame, and
    /// nothing on disk covered the window. `docs/CARAVAN.md` §8 guessed the
    /// turn or a detour; the capture says **neither**. Both sides push the
    /// same detour node, `(38508, 40620)`, on the same frame; both turn
    /// through the same eight bearings at the same rate, because
    /// `avg_speed` decays 3/4 a frame on both. On sim-frame 6206 the turn
    /// leaves 38.9° owed on both, under `move_step`'s 45° gate, so both
    /// compute the same step and the same landing point — `(38750, 40508)`
    /// — and the original **does not take it**. That point is a tile north
    /// of the one the caravan stands on, and `move_step@005faf30`
    /// (`005fb7c1`–`005fb7fd`) asks `UnitData::invalid_loc` about any step
    /// that changes tile: the city's own footprint refuses it, the step is
    /// dropped whole, and the unit turns another 24° and goes on
    /// sim-frame 6207 through a tile it may have.
    ///
    /// It is the last of the four things `docs/MOVEMENT.md`'s `move_step`
    /// section listed as read and not modelled, and it is the whole of the
    /// residue: every field below is the original's, and East Indies' word
    /// went **6207 → 6353**.
    ///
    /// Nothing here is installed. The simulation is built from run54's
    /// start and driven forward 6,213 frames; run65's own blocks are only
    /// ever read.
    #[test]
    fn run65_s_window_is_the_original_s_unit_for_unit() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r65)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run65-islands-caravanturn.txt"),
        ) else {
            eprintln!("skipping: no run54/run65 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);

        let text65 = std::fs::read_to_string(&r65).unwrap();
        let l65 = Log::parse(&text65);
        let frames = l65.frame_states();
        let window: Vec<i64> = frames.iter().map(|f| f.n).collect();
        // The window, and then the **quit's own two blocks**: the log
        // writes a frame at `!quit` and one behind it, so 6220 and 6221
        // come free — seven frames past anything the window paid for.
        // They are read like the rest, but only as far as they parse: the
        // log is cut where the process went, so the caravan's eighteen
        // rows below are the window's own and nothing is owed by the tail.
        assert_eq!(
            window,
            (6196..=6213).chain([6220, 6221]).collect::<Vec<i64>>(),
            "run65's `DUMP_ALL` window, plus the two blocks the quit writes"
        );
        let last = *window.last().expect("the window");

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let mut compared = 0usize;
        let mut caravan_rows = 0usize;
        let mut queue_rows = 0usize;
        let mut unmatched = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        // **`unit_masks & 8`, the verified-line bit** — the widening the
        // MOVEMENT docs-versus-code pass asked for (2026-09-05, R6/R7).
        // The dump has printed it all along and nothing compared it.
        //
        // It is `sim::Unit::line_ok`, and this crate has exactly one
        // reader of it, on the hot path: `do_move` re-paths when it is
        // clear. It disagreed on **335 of these 450 unit-frames** for one
        // item — the original carrying the bit *set* on standing units
        // where this crate carried it clear — and the cause was not the
        // refused step the row named but `clear_partial_path`, which
        // cleared the bit here and touches no mask at all in the original
        // (`docs/MOVEMENT.md`, "The verified line's lifecycle").
        //
        // At zero it is an ordinary row of the labelled comparison, and
        // that is what it is now; `line_ok_rows` survives as the coverage
        // count, so the row cannot pass by not being compared.
        let mut line_ok_rows = 0usize;
        // `Built::tick` stamps the frame it is about to run, so after the
        // tick with `f` the counter reads `f + 1` and the state is that
        // `FRAME` block's — the same alignment run39's scout test uses.
        for f in 0..last {
            built.tick();
            let Some(fr) = frames.iter().find(|fr| fr.n == f + 1) else {
                continue;
            };
            for them in &fr.units {
                // Gaia's animals are re-seated from the dump every traced
                // frame (`Sim::reseat_animal`), so they are not this
                // crate's prediction; the players' units are.
                if !(0..8).contains(&them.who) {
                    continue;
                }
                let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                    continue;
                };
                let Some(u) = built.sim.unit_by_o(who, o) else {
                    unmatched += 1;
                    continue;
                };
                let un = &built.sim.units[u];
                if (them.who, them.o) == (1, 18) {
                    caravan_rows += 1;
                }
                let mut row = |name: &str, ours: i64, theirs: Option<i64>| {
                    let Some(theirs) = theirs else { return };
                    compared += 1;
                    if ours != theirs && wrong.len() < 12 {
                        wrong.push(format!(
                            "frame {}: {}/{} {name} ours {ours} theirs {theirs}",
                            f + 1,
                            them.who,
                            them.o
                        ));
                    }
                };
                row("x", i64::from(un.pos.x), Some(them.pos.x));
                row("y", i64::from(un.pos.y), Some(them.pos.y));
                if them.unit_masks.is_some() {
                    line_ok_rows += 1;
                }
                row(
                    "line_ok",
                    i64::from(un.line_ok),
                    them.unit_masks.map(|m| i64::from(m & 8 != 0)),
                );
                // `UnitData::angle` is the **heading** `set_angle` writes,
                // not the facing: `move_step` hands it the bearing at the
                // top of the function and only `Guy::do_turn` moves guy 0
                // toward it. The facing is run64's test's business.
                row("angle", i64::from(un.movement.heading.0), them.angle);
                row("orders_x", i64::from(un.orders_pos.x), them.orders_x);
                row("orders_y", i64::from(un.orders_pos.y), them.orders_y);
                row("tolerance", i64::from(un.tolerance), them.tolerance);
                row(
                    "path length",
                    un.path.len() as i64,
                    Some(them.path.len() as i64),
                );
                // The stack whole — point, tolerance and flag byte, every
                // slot. It is what says the caravan walks the road's own
                // twenty-six nodes and not a plan of its own.
                for (slot, (ours, theirs)) in un.path.iter().zip(them.path.iter()).enumerate() {
                    row(
                        &format!("path[{slot}].x"),
                        i64::from(ours.to.x),
                        Some(theirs.to.0),
                    );
                    row(
                        &format!("path[{slot}].y"),
                        i64::from(ours.to.y),
                        Some(theirs.to.1),
                    );
                    row(
                        &format!("path[{slot}].tolerance"),
                        i64::from(ours.tolerance),
                        Some(theirs.tolerance),
                    );
                    row(
                        &format!("path[{slot}].flags"),
                        i64::from(ours.flags),
                        Some(theirs.flags),
                    );
                }
            }
            // **And the build queues, whole** — the widening this window
            // was re-read for. The AI's Market `1/2013` is one `MERCHANT`
            // deep on every block of the window, and its `job_counter`
            // climbs a hundred a frame from the caravan's own hand-over on
            // 6164 to the merchant's on 6353; nothing in this crate
            // trained one until `num_rare_resources_seen` stopped
            // answering zero (`docs/ECONOMY.md`, "The rares a leader has
            // seen"), so this is what says the *count* the script asked
            // for was right and not merely non-zero.
            //
            // The record's own trap is in the slot behind it: `queued` is
            // 1 and slot 1 reads `type 61` too, because `unqueue` shifts
            // the array down over the caravan and leaves the vacated tail
            // standing. Only the first `queued` slots are read, for
            // [`compare`]'s reason — the tail is not state either side
            // owns.
            for them in &fr.builds {
                let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                    continue;
                };
                let (Some(b), Some(q)) = (built.sim.building_by_o(who, o), them.queued) else {
                    continue;
                };
                let ours = &built.sim.buildings[b].queue;
                let mut row = |name: String, ours: i64, theirs: i64| {
                    compared += 1;
                    if ours != theirs && wrong.len() < 12 {
                        wrong.push(format!(
                            "frame {}: {}/{} {name} ours {ours} theirs {theirs}",
                            f + 1,
                            them.who,
                            them.o
                        ));
                    }
                };
                let mine = ours.items.len() as i64;
                row("queued".into(), mine, q);
                if mine != q {
                    continue;
                }
                for (k, theirs) in them.queue.iter().take(q as usize).enumerate() {
                    let item = &ours.items[k];
                    let id = item.tech.unwrap_or_else(|| built.unit_tree[item.ty]);
                    let ty = built.type_index.get(id).copied().unwrap_or(-1);
                    row(format!("queue[{k}].type"), i64::from(ty), theirs.ty);
                    row(
                        format!("queue[{k}].job_counter"),
                        i64::from(item.job_counter),
                        theirs.job_counter,
                    );
                    queue_rows += 1;
                }
            }
        }
        eprintln!(
            "run65 window: {compared} fields over 20 blocks, {caravan_rows} of them \
             the caravan's, {queue_rows} queue entries, {unmatched} of the dump's \
             units this crate has no unit for"
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 4_000 && caravan_rows >= 18,
            "the window's own rows: {compared} fields and {caravan_rows} caravan \
             frames — a capture with neither is the wrong file"
        );
        assert!(
            queue_rows >= 18,
            "the Market's merchant over the window's eighteen blocks: \
             {queue_rows} queue entries"
        );
        assert!(wrong.is_empty(), "run65's window parted: {wrong:?}");

        // The verified-line bit's coverage. `wrong` above carries any
        // disagreement now; what is pinned here is that the bit is still
        // being *read* on 450 unit-frames, so the row cannot pass by
        // going quiet.
        eprintln!("run65 `unit_masks & 8`: {line_ok_rows} rows compared");
        assert!(
            line_ok_rows >= 400,
            "the window prints `unit_masks` on {line_ok_rows} unit-frames; \
             a capture with fewer is the wrong file"
        );
    }

    /// **run66 — the merchant's whole walk, every unit, every frame**
    /// (2026-09-02, item 183).
    ///
    /// run54's game to 6,620 frames with the **cheap** per-frame dump
    /// narrowed to `[6340, 6600)` — 260 blocks at run39's `[End Frame]`
    /// detail, 89 MB, four minutes, against run59's thirteen for a window
    /// a tenth the size. `rngcmp.py rontrace-run54.log rontrace-run66.log`:
    /// **6,621 frames, zero differing**, so it is run54's game and the
    /// sixth capture in a row for which a window costs the stream nothing.
    ///
    /// **What it was booked for.** East Indies' word parted at 6570 on a
    /// collision the original does not have: the AI Merchant `1/19`,
    /// two hundred frames into its walk to a `CITRUS`, stood blocked
    /// twice where the original walked on. Nothing on disk covered the
    /// frame — run64 and run65's windows end at 6221 — and the geometry
    /// was a knife edge: the merchant's `BLOCK_RADIUS 2` block passes two
    /// standing citizens with 160 units of clearance against a 144-unit
    /// block sum, so a few units either way decides it.
    ///
    /// **What it settled** is `docs/COLLISION.md` §4.2's fast path, and
    /// the lesson is that it is not an optimisation. With `nocoll` clear
    /// and a proposal exactly one cell away on one axis,
    /// `CollCheck::collide_here@00682540` sweeps the **leading edge** —
    /// the row or column the block is entering — and nothing else: a
    /// strict subset of the parity-filtered disc, so it stops at a
    /// *different* first hit cell. On sim-frame 6570 the merchant's own
    /// cell is `(721, 786)` and its proposal `(721, 785)`, so the sweep
    /// is the row `y = 783` from `x = 719`: the first cell is `1/11`'s
    /// north-east corner, the disc probe's is `1/2`'s two to the right,
    /// and §4.3's corner rule is decided on the cell. Against `1/11` the
    /// merchant's own north-west corner is the opposite diagonal —
    /// `will_be_corner 1` against `is_corner 5`, a difference of exactly
    /// 4 — and the two slip past; against `1/2` the merchant's corner is
    /// 0 and the collision is hard. One frame later the proposal is one
    /// cell **west**, the sweep is the column `x = 718`, `1/11` is
    /// square on it rather than cornered, and the original collides —
    /// with `collide_o 11`, which is what named the cell.
    ///
    /// Every row below is the original's: 12,094 position fields — every
    /// unit on every block as far as the word, and `1/19` alone for all
    /// 261, so its walk, its collision, its centre snap and its recovery
    /// are pinned past the frame the stream parts on. East Indies' word
    /// went **6570 → 6571**.
    #[test]
    fn run66_s_window_is_the_original_s_unit_for_unit() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r66)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run66-islands-merchantwalk.txt"),
        ) else {
            eprintln!("skipping: no run54/run66 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);

        let text66 = std::fs::read_to_string(&r66).unwrap();
        let l66 = Log::parse(&text66);
        let frames = l66.frame_states();
        let window: Vec<i64> = frames.iter().map(|f| f.n).collect();
        // `[6340, 6600)` plus the two blocks the `!quit` writes — the same
        // free tail run65's window has.
        assert_eq!(
            window,
            (6340..6600).chain([6621]).collect::<Vec<i64>>(),
            "run66's frame window, plus the block the quit writes"
        );
        let last = *window.last().expect("the window");

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.sim.trace_phases = true;
        let mut compared = 0usize;
        let mut merchant_rows = 0usize;
        let mut unmatched = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..last {
            built.tick();
            let Some(fr) = frames.iter().find(|fr| fr.n == f + 1) else {
                continue;
            };
            for them in &fr.units {
                if !(0..8).contains(&them.who) {
                    continue;
                }
                let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                    continue;
                };
                let Some(u) = built.sim.unit_by_o(who, o) else {
                    unmatched += 1;
                    continue;
                };
                let merchant = (them.who, them.o) == (1, 19);
                if merchant {
                    merchant_rows += 1;
                } else if f + 1 > LONG_WORD_EAST_INDIES {
                    continue;
                }
                // **The closing block is not a frame state, so no unit
                // position is compared in it** (item 188, closed
                // 2026-09-03). It was excepted for `0/5` alone while that
                // read as a divergence; run68's ordinary block 6621 says
                // this crate has `0/5` exactly right, and the closing
                // block is the odd one out. Dump against dump: run66's
                // and run67's closing blocks each agree with run68's
                // ordinary 6621 for **130 of 131 units** and hold the
                // 6620 value for `0/5` — the same single unit in two
                // independent captures. `docs/ORACLE.md` carries it; the
                // harness simply does not score the block.
                if f + 1 == 6621 {
                    continue;
                }
                let un = &built.sim.units[u];
                let mut row = |name: &str, ours: i64, theirs: Option<i64>| {
                    let Some(theirs) = theirs else { return };
                    compared += 1;
                    if ours != theirs && wrong.len() < 24 {
                        wrong.push(format!(
                            "frame {}: {}/{} {name} ours {ours} theirs {theirs}",
                            f + 1,
                            them.who,
                            them.o
                        ));
                    }
                };
                row("x", i64::from(un.pos.x), Some(them.pos.x));
                row("y", i64::from(un.pos.y), Some(them.pos.y));
            }
        }
        eprintln!(
            "run66 window: {compared} fields over {} blocks, {merchant_rows} of \
             them the merchant's, {unmatched} of the dump's units this crate has \
             no unit for",
            window.len()
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 13_500 && merchant_rows >= 245,
            "the window's own rows: {compared} fields and {merchant_rows} of the \
             merchant's — a capture with neither is the wrong file"
        );
        // **The merchant's own walk, whole**, and everything else as far as
        // the word. Past the word the stream has parted — the crew figure's
        // draw `docs/COLLISION.md` §9 names — so the units downstream of it
        // are not this crate's prediction any more; `1/19` is, for all 260
        // blocks.
        assert!(wrong.is_empty(), "run66's window parted: {wrong:?}");
    }

    /// **run68 — the whole record, every unit, 120 blocks up to the word**
    /// (2026-09-03, items 188 and 189).
    ///
    /// run54's game with the cheap window over `[6595, 6730)` and `GUYS=4`
    /// — run67's recipe with the window moved and widened:
    ///
    /// ```text
    /// DETAIL_END="MISC,UNITS=3,BUILDS=7,CITIES=5,GUYS=4,DEATHS=1,LEADERS=1" \
    /// FRAME_WINDOW="6595 6730" SETTLE_MIN=20000000 POLL_MAX=60 \
    /// zsh tools/gamelog/longtrace.sh 68 6745 islands-citizenword 18
    /// ```
    ///
    /// **Five minutes and 83 MB**, 136 blocks. `rngcmp.py rontrace-run54.log
    /// rontrace-run68.log`: **6,746 frames, zero differing** — the eighth
    /// capture in a row for which a window costs the stream nothing.
    ///
    /// It was booked for two items and answered both, plus one nobody had
    /// asked:
    ///
    /// - **Item 188 was a non-issue, and the quit block is why.** The human
    ///   citizen `0/5` walks identically on both sides for its whole
    ///   journey — order on block 6607, first step on 6608, `(4860, 5172)`
    ///   on 6621. What said otherwise was run66's *closing* block, and it
    ///   is not a frame state: dump against dump, run66's and run67's
    ///   closing blocks each agree with this capture's **ordinary** 6621
    ///   for 130 of 131 units and hold the 6620 value for `0/5` alone.
    ///   Two independent captures, the same single unit — so the harness
    ///   does not compare the closing block, and `docs/ORACLE.md` carries
    ///   the fact.
    /// - **Item 189 was the merchant's arrival**, and it is closed. The
    ///   first field to part was `1/19`'s `orders_x/y` on **6714**, a
    ///   frame ahead of the draw stream's own 6715: the merchant reaches
    ///   its `CITRUS` and runs `find_merchant_spot`'s ring, which no
    ///   capture had ever reached (`docs/MERCHANT.md` §3, §6). The ring
    ///   agreed — both sides take tile `(168, 192)` — and what did not was
    ///   the queue position of the walk it orders; see
    ///   [`LONG_WORD_EAST_INDIES`]. With it in front of the cast the whole
    ///   window holds to **6718**, which is item 191's unit.
    /// - **`stance` is 1 on every unit here and 0 on every unit there**,
    ///   from the first block of the window — 2,700 rows, and no capture
    ///   had ever compared it on a unit this crate created. `Unit::init`
    ///   switches five ways on `get_stance_type` and reads the leader's
    ///   options (`00612100:282–309`); `Unit::new` writes a flat 1, and a
    ///   unit stood up *from* a dump takes the dump's own value, which is
    ///   why nothing saw it. Item 190, and it is excepted by name here.
    ///
    /// **117,126 fields** over the window, and the widening ledger
    /// (`crate::ledger`) is what named `stance`, `idle`, `path_recursion`
    /// and the collision block as the ones a single capture was carrying.
    #[test]
    fn run68_s_window_is_every_unit_s_whole_record_to_the_word() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r68)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run68-islands-citizenword.txt"),
        ) else {
            eprintln!("skipping: no run54/run68 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);

        let text68 = std::fs::read_to_string(&r68).unwrap();
        let l68 = Log::parse(&text68);
        let frames = l68.frame_states();
        let window: Vec<i64> = frames.iter().map(|f| f.n).collect();
        assert_eq!(
            window,
            (6595..6730).chain([6746]).collect::<Vec<i64>>(),
            "run68's frame window, plus the block the quit writes"
        );
        // The first field to part, and there is none inside the window
        // any more: every unit's whole record agrees on all 135 blocks up
        // to the last one this dump carries, with no exception but
        // `stance`. It was 6714 — `1/19`'s `orders_x/y` on the merchant's
        // arrival — then **6718**, `1/13`'s own position thirty-two frames
        // after item 191's two middle waypoints. The waypoints were the
        // collision index's missing repaint (`docs/COLLISION.md` §2.2) and
        // the position went with them, so this is now the window's end.
        const FIRST_FIELD_PARTING: i64 = 6730;
        const {
            assert!(
                FIRST_FIELD_PARTING < LONG_WORD_EAST_INDIES,
                "a field diff sees the divergence before the draw stream does"
            )
        };

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        built.frame_guys.clear();
        let mut compared = 0usize;
        let mut crew_rows = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..*window.last().expect("the window") {
            built.tick();
            let Some(fr) = frames.iter().find(|fr| fr.n == f + 1) else {
                continue;
            };
            // **The closing block is not a frame state** — see the head of
            // this test. It is skipped rather than compared at `n − 1`,
            // because it is neither: 130 of its 131 units are block `n`.
            if f + 1 >= FIRST_FIELD_PARTING {
                continue;
            }
            for them in &fr.units {
                // Gaia's units are re-seated from the dump every traced
                // frame, so they are not this crate's prediction.
                if !(0..8).contains(&them.who) {
                    continue;
                }
                let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                    continue;
                };
                let Some(u) = built.sim.unit_by_o(who, o) else {
                    continue;
                };
                let un = &built.sim.units[u];
                let mut rows: Vec<(&str, i64, Option<i64>)> = vec![
                    ("x", i64::from(un.pos.x), Some(them.pos.x)),
                    // Item 190: `Unit::init@00612100:282–309` switches five
                    // ways on the type's stance kind and reads the leader's
                    // options (`sim::stance`). It was excepted here while
                    // `Unit::new`'s flat 1 stood.
                    ("stance", i64::from(un.stance), them.stance),
                    ("y", i64::from(un.pos.y), Some(them.pos.y)),
                    ("angle", i64::from(un.movement.heading.0), them.angle),
                    ("orders_x", i64::from(un.orders_pos.x), them.orders_x),
                    ("orders_y", i64::from(un.orders_pos.y), them.orders_y),
                    ("tolerance", i64::from(un.tolerance), them.tolerance),
                    (
                        "path_recursion",
                        i64::from(un.path_recursion),
                        them.path_recursion,
                    ),
                    ("idle", i64::from(un.idle), them.idle),
                    ("collide", i64::from(un.collide), them.collide),
                    ("collide_o", i64::from(un.collide_o), them.collide_o),
                    ("collide_who", i64::from(un.collide_who), them.collide_who),
                    ("collide_guy", i64::from(un.collide_guy), them.collide_guy),
                    ("safe", i64::from(un.safe), them.safe),
                    (
                        "path length",
                        un.path.len() as i64,
                        Some(them.path.len() as i64),
                    ),
                ];
                // The stack whole — point, tolerance and flag byte, every
                // slot, labelled by index so a parted waypoint names
                // itself.
                //
                // **The stack is compared whole, and nothing is
                // excepted.** `1/13` was, until 2026-09-03: from block
                // 6686 its `path[2].y` read 38712 here against 38760 and
                // its `path[3].x` 40584 against 40536, two middle
                // waypoints one 48-grid step each. That was item 191 on
                // this map, and it closed with Great Lakes' — the
                // collision index's sixty-fourth-frame repaint
                // (`docs/COLLISION.md` §2.2). The exception is gone rather
                // than kept, so a route that parts again fails here.
                let labels: Vec<String> = (0..un.path.len().min(them.path.len()))
                    .flat_map(|slot| {
                        ["x", "y", "tol", "flags"]
                            .map(|f| format!("path[{slot}].{f}"))
                            .into_iter()
                    })
                    .collect();
                for (slot, (ours, theirs)) in un.path.iter().zip(them.path.iter()).enumerate() {
                    rows.push((&labels[slot * 4], i64::from(ours.to.x), Some(theirs.to.0)));
                    rows.push((
                        &labels[slot * 4 + 1],
                        i64::from(ours.to.y),
                        Some(theirs.to.1),
                    ));
                    rows.push((
                        &labels[slot * 4 + 2],
                        i64::from(ours.tolerance),
                        Some(theirs.tolerance),
                    ));
                    rows.push((
                        &labels[slot * 4 + 3],
                        i64::from(ours.flags),
                        Some(theirs.flags),
                    ));
                }
                for (n, g) in them.guys.iter().enumerate() {
                    let Some(og) = built.sim.units[u].guys.get(n).copied() else {
                        continue;
                    };
                    let un = &built.sim.units[u];
                    // The same three fallbacks run67 established: guy 0's
                    // `des_angle` is the heading, and a trackless crew
                    // figure's is guy 0's facing.
                    let (body, facing, des, des_angle) = match og.follow {
                        Some(b) => (b.body, b.facing, b.des, b.des_angle),
                        None if n == 0 => (
                            un.movement.body,
                            un.movement.facing,
                            un.pos,
                            un.movement.heading,
                        ),
                        None => (
                            un.movement.body,
                            un.movement.facing,
                            un.pos,
                            un.movement.facing,
                        ),
                    };
                    if n >= sim::anim::SQUAD_SIZE {
                        crew_rows += 1;
                    }
                    let track = og.follow.map_or((0, 0), |b| b.track);
                    for r in [
                        ("g.x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                        ("g.y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                        ("g.angle", i64::from(facing.0), g.angle),
                        ("g.des_x", i64::from(des.x), g.des.map(|p| p.x)),
                        ("g.des_y", i64::from(des.y), g.des.map(|p| p.y)),
                        ("g.des_angle", i64::from(des_angle.0), g.des_angle),
                        ("g.cur_anim", i64::from(og.anim), g.cur_anim),
                        ("g.cur_time", i64::from(og.cur_time), g.cur_time),
                        ("g.end_time", i64::from(og.end_time), g.end_time),
                        ("g.last_time", i64::from(og.last_time), g.last_time),
                        ("g.gpiece", i64::from(og.gpiece), g.gpiece),
                        ("g.stopped", i64::from(og.stopped), g.stopped),
                        ("g.track_dx", i64::from(track.0), g.track.map(|t| t.0)),
                        ("g.track_dy", i64::from(track.1), g.track.map(|t| t.1)),
                        ("g.last_speed", i64::from(body.last_speed), g.last_speed),
                        ("g.avg_speed", i64::from(body.avg_speed), g.avg_speed),
                    ] {
                        rows.push(r);
                    }
                }
                for (name, ours, theirs) in rows {
                    let Some(theirs) = theirs else { continue };
                    compared += 1;
                    if ours != theirs && wrong.len() < 16 {
                        wrong.push(format!(
                            "frame {}: {}/{} {name} ours {ours} theirs {theirs}",
                            f + 1,
                            them.who,
                            them.o
                        ));
                    }
                }
            }
        }
        eprintln!(
            "run68: {compared} fields over the window's first {} blocks, {crew_rows} crew rows",
            FIRST_FIELD_PARTING - 6595
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 100_000 && crew_rows >= 300,
            "the window's own rows: {compared} fields and {crew_rows} crew figures — \
             a capture without the `GUYS=4` block is the wrong file"
        );
        assert!(wrong.is_empty(), "run68's window parted: {wrong:?}");
    }

    /// **run67 — every figure's whole record over the merchant's
    /// collision** (2026-09-02, item 185).
    ///
    /// run54's game to 6,620 frames with the cheap per-frame dump narrowed
    /// to `[6545, 6605)` and its `GUYS` detail raised from 2 to **4** —
    /// 61 blocks, 43 MB, four minutes. `rngcmp.py rontrace-run54.log
    /// rontrace-run67.log`: **6,621 frames, zero differing**, the seventh
    /// capture in a row for which a window costs the stream nothing.
    ///
    /// **`GUYS=4` is the finding that booked it.** `GuyData::log_data`
    /// switches detail four times (`005de6c0`, the calls to the log's
    /// vslot `0x28`), and the fourth block is the whole of the record:
    /// `des_x`, `des_y`, `des_angle`, `cur_time`, `end_time`, `last_time`,
    /// `cur_anim`, `stopped`, `guy_flags`, `guy_num`, `gpiece`, `track_dx`
    /// and `track_dy`. Nothing before this had read a crew figure's clock
    /// outside a `DUMP_ALL` window, and `DUMP_ALL` is what run65 paid
    /// thirty-seven minutes and 1.19 GB for eighteen frames of. A whole
    /// record over sixty frames is a **category**, not a window — the
    /// third shape, beside run60's narrow window and run66's cheap block.
    ///
    /// **What it settled**, and all three are one mechanism — the crew
    /// loop that `Guy::set_angle` and `Guy::set_new_location` share:
    ///
    /// - **`Unit::set_angle` rewrites the crew's `des`**, from guy 0's own
    ///   point and the **heading** `find_angle` has just returned. It is
    ///   the third row of `docs/MOVEMENT.md`'s writer table and had stood
    ///   "not modelled" since the table was written. It fires at the *top*
    ///   of `move_step`, ahead of the collision block, so a figure that
    ///   walked exactly onto its destination last frame is off it again
    ///   before the blocked stand asks it to idle: 720,896 of a turn moves
    ///   a (-48, -192) track by one unit on each axis, `Guy::set_anim`'s
    ///   walking-guy early return takes it, and it does not roll.
    /// - **The cell-centre snap teleports the crew.**
    ///   `Unit::set_new_location`'s `param_3` is handed on as
    ///   `Guy::set_new_location(guy 0, pos, 1)`, whose crew loop runs
    ///   `set_angle(crew, des_angle, 1)` and `set_new_location(crew, des,
    ///   1)` — the figure is *put* on its new offset with its leader's
    ///   angle rather than left to walk after it. Block 6572 is the
    ///   record: `1/19`'s figure on (34464, 37595) with `angle` equal to
    ///   its driver's to the digit.
    /// - **The walk slot is the asked guy's own average speed**
    ///   (`005db438`, `this->field_0x84`). A tracked figure is paid
    ///   `(get_speed * 11) / 8` a frame to keep station, so it averages
    ///   eleven eighths of its leader's base and **jogs where its leader
    ///   walks**: `cur_anim 9` against 8 on every block of this window.
    ///   The slot, not the category, is what `Guy::move`'s arrival arm
    ///   tests (`== CHAR_WALK`), so reading guy 0's cost a draw on every
    ///   arrival a crew figure made.
    ///
    /// East Indies' long word **6571 -> 6574**, and **6574 -> 6715** when
    /// item 186 took the window's last exception away: with the second
    /// Merchant walking at its own rare this compares every unit of every
    /// block, **28,890 fields** against 14,910.
    #[test]
    fn run67_s_window_is_every_figure_s_whole_record() {
        let Some(inst) = install() else { return };
        let (Some(path), Some(sib), Some(tr), Some(r67)) = (
            dump("gamelog-run54-islands-24k-trace.txt"),
            dump("gamelog-run38-islands-start.txt"),
            trace("rontrace-run54.log"),
            dump("gamelog-run67-islands-crewclocks.txt"),
        ) else {
            eprintln!("skipping: no run54/run67 capture (set RON_GAMELOG_DIR)");
            return;
        };
        let loaded = crate::load::load(&inst).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let sib_text = std::fs::read_to_string(&sib).unwrap();
        let log = Log::parse(&text);
        let sib_log = Log::parse(&sib_text);
        let sib_init = sib_log.initial().expect("run38 is a start dump");
        let mut init = log.initial().unwrap();
        borrow_from_siblings(&mut init, &[&sib_init]);
        borrow_pasture(&mut init, &tr);

        let text67 = std::fs::read_to_string(&r67).unwrap();
        let l67 = Log::parse(&text67);
        let frames = l67.frame_states();
        let window: Vec<i64> = frames.iter().map(|f| f.n).collect();
        assert_eq!(
            window,
            (6545..6605).chain([6621]).collect::<Vec<i64>>(),
            "run67's frame window, plus the block the quit writes"
        );
        let last = *window.last().expect("the window");

        let mut built = build_sim(&loaded, &init, Tuning::RON);
        // The window's own clocks are never installed: this is the check.
        built.frame_guys.clear();
        let mut compared = 0usize;
        let mut crew_rows = 0usize;
        let mut unmatched = 0usize;
        let mut wrong: Vec<String> = Vec::new();
        for f in 0..last {
            built.tick();
            let Some(fr) = frames.iter().find(|fr| fr.n == f + 1) else {
                continue;
            };
            for them in &fr.units {
                // Gaia's are re-seated from the dump every traced frame
                // (`Sim::reseat_animal`), so their figures are not this
                // crate's prediction; the players' are.
                if !(0..8).contains(&them.who) {
                    continue;
                }
                let (Ok(who), Ok(o)) = (u8::try_from(them.who), i16::try_from(them.o)) else {
                    continue;
                };
                let Some(u) = built.sim.unit_by_o(who, o) else {
                    unmatched += 1;
                    continue;
                };
                // **Every unit, every block, no exception** since item
                // 186 (2026-09-02). The window used to straddle the word:
                // only `1/19` was compared past 6574, and the second
                // Merchant `1/20` was excepted by name from the frame it
                // was born on. Both exceptions are gone — the word is
                // past the whole capture, quit block included.
                assert!(
                    last < LONG_WORD_EAST_INDIES,
                    "run67's window has fallen outside the word again"
                );
                for (n, g) in them.guys.iter().enumerate() {
                    let Some(ours) = built.sim.units[u].guys.get(n).copied() else {
                        continue;
                    };
                    let un = &built.sim.units[u];
                    let (body, facing, des, des_angle) = match ours.follow {
                        Some(b) => (b.body, b.facing, b.des, b.des_angle),
                        // A **trackless** crew figure has no body of its
                        // own, and this crate keeps none — but the original
                        // still writes it a `des` and a `des_angle`, because
                        // the crew loop's `track != 0` test gates only the
                        // rotation. Its point is guy 0's own, and its angle
                        // is guy 0's **facing**: the last of the three
                        // writers to run on a walking frame is
                        // `Guy::set_new_location`, which passes
                        // `guy0->angle`, and `Guy::do_turn`'s recursion
                        // passes the same. `Unit::set_angle`'s heading is
                        // overwritten by both.
                        None if n == 0 => (
                            un.movement.body,
                            un.movement.facing,
                            un.pos,
                            // Guy 0's own `des_angle` is the **heading**:
                            // `Unit::set_angle` writes `UnitData::angle`
                            // and hands the same value straight to
                            // `Guy::set_angle(guy 0, …)`, which is
                            // `docs/MOVEMENT.md`'s "the same value again".
                            un.movement.heading,
                        ),
                        None => (
                            un.movement.body,
                            un.movement.facing,
                            un.pos,
                            un.movement.facing,
                        ),
                    };
                    if n >= sim::anim::SQUAD_SIZE {
                        crew_rows += 1;
                    }
                    let track = ours.follow.map_or((0, 0), |b| b.track);
                    // **`GUYS=4` prints the speed pair as well**, and
                    // until item 210 no capture compared it. It is the
                    // walk slot's own input: a tracked figure paid
                    // `(get_speed · 11) / 8` a frame averages eleven
                    // eighths of its leader's base and jogs where the
                    // leader walks, and the slot — not the category — is
                    // what `Guy::move`'s arrival draw tests. This window
                    // is where that arithmetic is checked against the
                    // original's own numbers.
                    let rows: [(&str, i64, Option<i64>); 17] = [
                        ("x", i64::from(body.pos.x), g.pos.map(|p| p.x)),
                        ("y", i64::from(body.pos.y), g.pos.map(|p| p.y)),
                        ("angle", i64::from(facing.0), g.angle),
                        ("des_x", i64::from(des.x), g.des.map(|p| p.x)),
                        ("des_y", i64::from(des.y), g.des.map(|p| p.y)),
                        ("des_angle", i64::from(des_angle.0), g.des_angle),
                        ("cur_anim", i64::from(ours.anim), g.cur_anim),
                        ("cur_time", i64::from(ours.cur_time), g.cur_time),
                        ("end_time", i64::from(ours.end_time), g.end_time),
                        ("last_time", i64::from(ours.last_time), g.last_time),
                        ("gpiece", i64::from(ours.gpiece), g.gpiece),
                        ("stopped", i64::from(ours.stopped), g.stopped),
                        ("guy_num", i64::try_from(n).unwrap_or(-1), g.guy_num),
                        ("track_dx", i64::from(track.0), g.track.map(|t| t.0)),
                        ("track_dy", i64::from(track.1), g.track.map(|t| t.1)),
                        ("last_speed", i64::from(body.last_speed), g.last_speed),
                        ("avg_speed", i64::from(body.avg_speed), g.avg_speed),
                    ];
                    for (name, ours, theirs) in rows {
                        let Some(theirs) = theirs else { continue };
                        compared += 1;
                        if ours != theirs && wrong.len() < 16 {
                            wrong.push(format!(
                                "frame {}: {}/{} guy {n} {name} ours {ours} theirs {theirs}",
                                f + 1,
                                them.who,
                                them.o
                            ));
                        }
                    }
                }
            }
        }
        eprintln!(
            "run67 figures: {compared} fields over {} blocks, {crew_rows} crew rows, \
             {unmatched} of the dump's units this crate has no unit for",
            window.len()
        );
        for w in &wrong {
            eprintln!("  {w}");
        }
        assert!(
            compared >= 32_000 && crew_rows >= 330,
            "the window's own rows: {compared} fields and {crew_rows} crew figures — \
             a capture without the `GUYS=4` block is the wrong file"
        );
        assert!(wrong.is_empty(), "run67's window parted: {wrong:?}");
    }
}
