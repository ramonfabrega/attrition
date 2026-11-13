//! The animation lengths, read from the install rather than from a dump —
//! `docs/ANIM.md` §3.1, `docs/FORMATS.md` "The animation file (`.BHa`)".
//!
//! `Guy::set_anim` ends by writing `end_time = animmgr.frames[packet
//! .action_ids[cur_anim]]`, and until now the sim took that table out of a
//! `DUMP_ALL` dump's `GUY` blocks: a length nothing played was a length
//! nothing knew. That is what left the bird without one — **no dump prints
//! owner 9** (`docs/SYNC.md` §3.9), so gaia's bird carried `gpiece = −1`,
//! every lookup failed and its animation never wrapped.
//!
//! The install says it outright, in three files that are all open data:
//!
//! - `Data/unit_graphics.xml` — `<UNIT name="HERDSHEEP-TYPE1">` and its
//!   `<ANIM name="CHAR_DEFAULT" file="Sheep Idle3"/>` children: the
//!   packet's `action_ids`, one row per slot. The `-TYPE<v>` suffix of a
//!   gaia entry **is** the variant `(seed + o) % 3` picks (§3).
//! - `Data/anim_graphics.xml` — `<ANIM name="Sheep Idle3" file=".\art\
//!   sheep_idle3.bha"/>`, under `<LOOPING>` or `<NONLOOPING>`: the name a
//!   slot cites, resolved to a file.
//! - `art/*.bha` — the animation itself, whose root node's key times give
//!   `AnimMgr::force_load`'s `times[]`, and with it `frames[] = round(times
//!   · 3 / 200)` — fifteen frames a second, **with a non-looping
//!   animation's last key dropped first** ([`game_frames`]). Which is why
//!   the section a row sits in is read alongside its file.
//!
//! Only the **gaia** types are read here. A player's unit needs
//! `GraphicPieces::get_unit_gpiece`'s tribe, age and gender walk to know
//! which `<UNIT>` entry it plays, and none of that is modelled; a gaia
//! type's is the `-TYPE<v>` suffix and nothing else. The dump stays the
//! source for everyone else, and
//! `crate::diff::tests::the_install_s_gaia_lengths_match_the_dump_s` is
//! what keeps the two honest: six lengths — three sheep, three fish — that
//! both oracles state.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::Install;

/// `(TypeIndex, variant, slot) → frames`, for the twelve gaia types.
pub type GaiaLengths = BTreeMap<(i32, u8, i8), u32>;

/// `gpiece → (slot → frames)` for every graphic piece the install's own
/// `<UNIT>` entries name — a player's units, where [`GaiaLengths`] covers
/// gaia's. A piece present here has its **whole** slot list known, so a
/// slot it omits is the packet's own missing slot.
pub type PieceLengths = BTreeMap<i32, BTreeMap<i8, u32>>;

/// `GraphicPieces::init_piece_ranges@008f70e0`, read straight out of the
/// executable — the layout of the unit half of the graphic-piece pool.
/// `get_unit_gpiece@0090c030` indexes it as
///
/// ```text
/// piece = (TypeIndex - 0x32)
///       + NUM_PIECES      * art_set     # 0..5, the tribe's UNIT_CONTINENT
///       + PER_AGE         * age_bracket # 0..2
///       + PER_GENDER      * female      # or `packed`, which shares the slot
///       + PER_CREW        * guy_num     # 0..3
/// ```
///
/// and the four strides nest exactly: `PER_AGE` is six art sets of
/// `NUM_PIECES`, `PER_GENDER` three ages of `PER_AGE`, `PER_CREW` two
/// genders of `PER_GENDER`, and `total_num_unit_pieces` is `0xc606` —
/// four crews of `PER_CREW` plus the six "over time" pieces the function
/// reaches by `total - 6 … total - 1`. `first_unit_piece` is zero.
///
/// Diff-backed: run12's own `GUY` blocks give the human scout `371` and
/// its dog `13043`, the AI's `19` and `12691`, and player 0's citizens
/// `352` (male) / `6688` (female) — which this arithmetic reproduces from
/// the two nations' `UNIT_CONTINENT` (Nubians `1 Arab`, British
/// `0 European`) and the `GUY.type` the same records carry.
const NUM_PIECES: i32 = 0x160;
const PER_AGE: i32 = 0x840;
const PER_GENDER: i32 = 0x18c0;
const PER_CREW: i32 = 0x3180;

/// The six unit art styles, by the index a nation's `UNIT_CONTINENT`
/// carries — `say_unit_art_style_name@006f02e0`'s switch, whose six arms
/// name six consecutive `internal_strings.xml` entries: `Europe` (the
/// empty style, written `DEFAULT` in the graphics file), `Arab`, `American`,
/// `Asian`, `NA` and `India`.
///
/// A `<UNIT>` whose style is none of these — the handful of `MERCHANT`
/// entries written `-NEUROPE-`, `-KOREAN-`, `-IROQUOIS-`, `-COLONIAL-`,
/// `-EINDIAN-` — is a name `get_unit_gpiece` can never build, so it holds
/// no piece and is skipped.
pub const STYLES: [&str; 6] = ["DEFAULT", "ARAB", "AMERICAN", "ASIAN", "NA", "INDIA"];

/// The `UnitAnim` slot each `<ANIM name="CHAR_…">` names. The indices are
/// `sim::units::anim`'s, which are the enum's (`rise.pdb`, type 0x46B1); a name
/// this table does not carry is skipped rather than guessed.
const SLOTS: &[(&str, i8)] = &[
    ("CHAR_DEFAULT", sim::units::anim::DEFAULT),
    ("CHAR_IDLE1", sim::units::anim::IDLE1),
    ("CHAR_IDLE2", sim::units::anim::IDLE2),
    ("CHAR_IDLE3", sim::units::anim::IDLE3),
    ("CHAR_GROUP_IDLE1", sim::units::anim::GROUP_IDLE1),
    ("CHAR_GROUP_IDLE2", sim::units::anim::GROUP_IDLE2),
    ("CHAR_GROUP_IDLE3", sim::units::anim::GROUP_IDLE3),
    ("CHAR_SLOG", sim::units::anim::SLOG),
    ("CHAR_WALK", sim::units::anim::WALK),
    ("CHAR_JOG", sim::units::anim::JOG),
    ("CHAR_ATTACKWALK", sim::units::anim::ATTACKWALK),
    ("CHAR_ATTACK1", sim::units::anim::ATTACK1),
    ("CHAR_ATTACK2", sim::units::anim::ATTACK2),
    ("CHAR_ATTACK3", sim::units::anim::ATTACK3),
    ("CHAR_ATTACKSPECIAL", sim::units::anim::ATTACKSPECIAL),
    ("CHAR_DEATH_STAB1", sim::units::anim::DEATH_STAB1),
    ("CHAR_DEATH_STAB2", sim::units::anim::DEATH_STAB2),
    ("CHAR_DEATH_SHOT1", sim::units::anim::DEATH_SHOT1),
    ("CHAR_DEATH_SHOT2", sim::units::anim::DEATH_SHOT2),
    ("CHAR_DEATH_SPLODED1", sim::units::anim::DEATH_SPLODED1),
    ("CHAR_DEATH_SPLODED2", sim::units::anim::DEATH_SPLODED2),
    ("CHAR_TURN_LEFT", sim::units::anim::TURN_LEFT),
    ("CHAR_TURN_RIGHT", sim::units::anim::TURN_RIGHT),
    ("CHAR_PACK", sim::units::anim::ANIM_PACK),
    ("CHAR_UNPACK", sim::units::anim::ANIM_UNPACK),
    ("CHAR_CHOP_WOOD", sim::units::anim::CHOP_WOOD),
    ("CHAR_WALK_WITH_WOOD", sim::units::anim::WALK_WITH_WOOD),
    ("CHAR_DUMP_WOOD", sim::units::anim::DUMP_WOOD),
    ("CHAR_WALK_TO_WOOD", sim::units::anim::WALK_TO_WOOD),
    ("CHAR_MINE_ORE", sim::units::anim::MINE_ORE),
    ("CHAR_WALK_WITH_ORE", sim::units::anim::WALK_WITH_ORE),
    ("CHAR_DUMP_ORE", sim::units::anim::DUMP_ORE),
    ("CHAR_WALK_TO_ORE", sim::units::anim::WALK_TO_ORE),
    ("CHAR_BUILD", sim::units::anim::BUILD),
    ("CHAR_REPAIR", sim::units::anim::ANIM_REPAIR),
    ("CHAR_SOW", sim::units::anim::SOW),
    ("CHAR_REAP", sim::units::anim::REAP),
    ("CHAR_FARM", sim::units::anim::ANIM_FARM),
];

/// The `<UNIT name="…">` prefix of each gaia `TypeIndex`, in the order the
/// enum gives them (`BASE_GAIATYPES = 0x192` upwards). The three birds and
/// the whale carry a `-TYPE0` only, which every variant then shares.
///
/// `FARMPIG` (0x195) and `FARMCHICKEN` (0x196) — the pasture's five,
/// `docs/SYNC.md` §3.6 — were left out while the missing `-TYPE2` looked
/// like a guess. It is not one: the `-TYPE0` fall-back below is the piece
/// pool's own, and in any case **both of their `-TYPE` entries name the
/// same three animation files** (`Pig Default`, `Pig Idle1`, `Pig Walk`;
/// `Chicken …`), so the variant cannot change a length. What they give is
/// `CHAR_DEFAULT` 90 and the walks 15 for the pig, 30 and 18 for the
/// chicken, with `CHAR_IDLE1..3` 90 for both.
///
/// These two rows were inert while a pasture animal carried no
/// `type_index` — `Sim::slot_length` keys a gaia type by the index, so the
/// lookup missed and the clock never wrapped. **Live since 2026-08-30**:
/// `sim::Sim::farm_add_animals` sets the index from the species the
/// capture's own trace names, and with the walk that goes with it run39's
/// early window moves 49/47 → 62/55 of its first 64 frames
/// (`docs/SYNC.md` §3.11,
/// `crate::diff::tests::run39_s_long_trace_says_where_the_second_map_s_word_parts`).
const GAIA_UNITS: &[(i32, &str)] = &[
    (0x192, "WILDBIRD"),
    (0x193, "FLOCKBIRD"),
    (0x194, "GULLBIRD"),
    (0x195, "FARMPIG"),
    (0x196, "FARMCHICKEN"),
    (0x197, "HERDHORSES"),
    (0x198, "HERDSHEEP"),
    (0x199, "HERDBISON"),
    (0x19a, "HERDBLACKBEAR"),
    (0x19b, "HERDFISH"),
    (0x19c, "HERDWHALES"),
    (0x19d, "HERDPEACOCKS"),
];

/// Every gaia type's `(variant, slot) → frames`, read from the install.
///
/// A file the install does not have, or one this reader cannot parse,
/// leaves its slot out — and a slot left out is the packet's own missing
/// slot, which `Guy::set_anim` gives the original's fallback of three
/// frames (`sim::units::anim::MISSING`). Returns an empty map when either XML is
/// unreadable, which is what a tables-only load gets.
pub fn gaia_lengths(install: &Install) -> GaiaLengths {
    let apath = install.data("anim_graphics.xml");
    let upath = install.data("unit_graphics.xml");
    let (Ok(atext), Ok(utext)) = (crate::read(&apath), crate::read(&upath)) else {
        return GaiaLengths::new();
    };
    let (Ok(adoc), Ok(udoc)) = (crate::parse(&apath, &atext), crate::parse(&upath, &utext)) else {
        return GaiaLengths::new();
    };
    let files = anim_files(&adoc);
    let units = unit_anims(&udoc);
    let mut frames: BTreeMap<String, u32> = BTreeMap::new();
    let mut out = GaiaLengths::new();
    for &(ty, prefix) in GAIA_UNITS {
        for v in 0..3u8 {
            // `-TYPE1` and `-TYPE2` are optional: a type that names only
            // `-TYPE0` plays it for all three variants, which is what the
            // piece pool's fall-back does.
            let name = format!("{prefix}-TYPE{v}");
            let rows = units
                .get(&name)
                .or_else(|| units.get(&format!("{prefix}-TYPE0")));
            let Some(rows) = rows else { continue };
            for (slot, anim) in rows {
                let Some((file, looping)) = files.get(&anim_key(anim)) else {
                    continue;
                };
                let n = match frames.get(file) {
                    Some(&n) => n,
                    None => {
                        let Some(n) = file_frames(install.root(), file, *looping) else {
                            continue;
                        };
                        frames.insert(file.clone(), n);
                        n
                    }
                };
                out.insert((ty, v, *slot), n);
            }
        }
    }
    out
}

/// One `<UNIT name="…">`'s place in the piece pool: the graphic's own name
/// and the four coordinates of [`NUM_PIECES`]' arithmetic.
///
/// The name is `{GRAPH}-{STYLE}-AGE{n}` with three optional tails —
/// `-PACKED`, `-CREW{k}` and `-FEMALE`. `PACKED` and `FEMALE` are the
/// **same** coordinate: `get_unit_gpiece` reaches the packed art through
/// its `packing` argument in place of the gender bit, and no shipped entry
/// carries both. The age digit is the age itself, not the bracket — `0`,
/// `3` and `5` for brackets 0, 1 and 2, which is `age < 5 ? age / 3 : 2`,
/// the same fold `get_unit_gpiece` applies to the leader's age.
struct PieceName<'a> {
    graph: &'a str,
    style: i32,
    age: i32,
    gender: i32,
    crew: i32,
}

impl<'a> PieceName<'a> {
    /// Splits a `<UNIT>` name, or `None` when it is not one the unit path
    /// can build — a gaia `-TYPE<v>`, or one of the styles [`STYLES`] does
    /// not name.
    fn parse(name: &'a str) -> Option<PieceName<'a>> {
        let (graph, rest) = name.split_once('-')?;
        let mut parts = rest.split('-');
        let style = parts.next()?;
        let style = STYLES.iter().position(|s| *s == style)? as i32;
        let age: i32 = parts.next()?.strip_prefix("AGE")?.parse().ok()?;
        let age = if age < 5 { age / 3 } else { 2 };
        let (mut gender, mut crew) = (0, 0);
        for tail in parts {
            match tail {
                "PACKED" | "FEMALE" => gender = 1,
                _ => crew = tail.strip_prefix("CREW")?.parse().ok()?,
            }
        }
        Some(PieceName {
            graph,
            style,
            age,
            gender,
            crew,
        })
    }

    /// The piece index of one `TypeIndex` playing this entry.
    fn piece(&self, type_index: i32) -> i32 {
        (type_index - 0x32)
            + NUM_PIECES * self.style
            + PER_AGE * self.age
            + PER_GENDER * self.gender
            + PER_CREW * self.crew
    }
}

/// **The six "over time" merchant pieces** (item 3369), in the order
/// `GraphicPieces::unit_node_from_gpiece@00901c30:174`–`198` names them
/// — six consecutive `internal_strings.xml` entries, `total_num_unit_pieces
/// − 6` first. They are the pieces `get_unit_gpiece@0090c030:104`–`140`
/// gives an unpacked merchant by the nation's `BUILD_CONTINENT`
/// (`sim::units::anim`'s stall arm); the name grammar cannot build them, so they
/// are matched whole. The install's other `-NEUROPE-`/`-KOREAN-` ages are
/// on no index and stay skipped.
pub const OVER_TIME: [&str; 6] = [
    "MERCHANT-KOREAN-AGE3",
    "MERCHANT-NEUROPE-AGE3",
    "MERCHANT-IROQUOIS-AGE0",
    "MERCHANT-IROQUOIS-AGE3",
    "MERCHANT-COLONIAL-AGE3",
    "MERCHANT-EINDIAN-AGE3",
];

/// `total_num_unit_pieces`, the first index past the unit pieces.
pub use sim::units::anim::TOTAL_UNIT_PIECES;

/// Every piece a `<UNIT>` entry is: an "over time" name's one index
/// ([`OVER_TIME`]), or the arithmetic's for each `TypeIndex` that plays
/// its `GRAPH`. `None` when it is neither.
fn entry_pieces(name: &str, by_graph: &BTreeMap<String, Vec<i32>>) -> Option<Vec<i32>> {
    if let Some(k) = OVER_TIME.iter().position(|n| *n == name) {
        return Some(vec![TOTAL_UNIT_PIECES - 6 + k as i32]);
    }
    let p = PieceName::parse(name)?;
    let types = by_graph.get(&p.graph.to_ascii_uppercase())?;
    Some(types.iter().map(|&ty| p.piece(ty)).collect())
}

/// `first_bird_piece`, the first of the gaia ("bird") pieces —
/// `GraphicPieces::init_piece_ranges@008f70e0` writes `0xea8d`, and
/// `num_bird_pieces` `0x24`: three variants for each of the twelve gaia
/// types `0x192..0x19e`, `get_type@00902540` reading a piece back as
/// `(piece - first_bird_piece) / 3 + 0x192`.
pub const FIRST_BIRD_PIECE: i32 = 0xea8d;

/// **A gaia entry's pieces** (item 4028): `{GRAPH}-TYPE{v}` is piece
/// `first_bird_piece + 3·(TypeIndex − 0x192) + v`.
/// `GraphicPieces::unit_node_from_gpiece@00901c30` builds exactly that name
/// for a piece past `first_bird_piece` — the type's `GRAPH` (`+0x88`),
/// `-`, `TYPE`, then `(piece − first_bird_piece) % 3` — and takes the
/// `<UNIT>` whose `name` equals it under `String::operator==@00a1f140`,
/// which compares without case (`_wcsicmp`). There is no `-TYPE0`
/// fall-back in the loader: a variant with no entry is a piece with no
/// node, and `init_unit_data@008f4290` returns `LIBERR_FILE_NOT_FOUND` for
/// it. Diff-backed by the start dumps: the sheep's 60063–60065 and the
/// fish's 60072–60074 are the pieces `GUY` prints for owner 9's herds.
fn bird_pieces(name: &str, graphs: &[String]) -> Option<Vec<i32>> {
    let (graph, v) = name.rsplit_once("-TYPE")?;
    let v: i32 = v.parse().ok().filter(|v| (0..3).contains(v))?;
    let pieces: Vec<i32> = graphs
        .iter()
        .enumerate()
        .filter_map(|(i, g)| {
            let ty = 0x32 + i as i32;
            ((0x192..0x19e).contains(&ty) && g.trim().eq_ignore_ascii_case(graph))
                .then_some(FIRST_BIRD_PIECE + 3 * (ty - 0x192) + v)
        })
        .collect();
    (!pieces.is_empty()).then_some(pieces)
}

/// `GRAPH` to the `TypeIndex`es that play it, keyed **upper case**. The
/// gaia records (`0x192` up) are left out: their pieces come from
/// `first_bird_piece`, not from the unit arithmetic ([`gaia_lengths`]).
///
/// **The name is matched without case** (item 3329): the one mixed-case
/// `GRAPH` the install ships, the Dutch caravan's `ArmedCaravan`
/// (`TypeIndex` 60, `CARADUTCH`), plays the `ARMEDCARAVAN-…` entries in
/// the original — run963's `1/14`, born on 1764, holds piece 10 and 12682,
/// its age-0 slot and its crew's — and a case-sensitive key left the type
/// with no piece, so ours fell back to `first_unit_piece`.
/// [`pivot_restrictions`] already matched without case.
fn graph_types(graphs: &[String]) -> BTreeMap<String, Vec<i32>> {
    let mut by_graph: BTreeMap<String, Vec<i32>> = BTreeMap::new();
    for (i, g) in graphs.iter().enumerate() {
        let ty = 0x32 + i as i32;
        if ty >= 0x192 {
            break;
        }
        by_graph
            .entry(g.trim().to_ascii_uppercase())
            .or_default()
            .push(ty);
    }
    by_graph
}

/// Every player unit piece's `(slot → frames)`, read from the install.
///
/// `graphs` is the `GRAPH` column of each unit record in file order — the
/// string `UnitType::init` keeps at `+0x88`, which is what
/// `get_unit_gpiece`'s name is built from — so record `i` is `TypeIndex`
/// `0x32 + i`. Records share a `GRAPH` (thirteen of the shipped 364 do),
/// and a shared one gives each of its types its own piece with the same
/// animations, which is what a lookup by *name* does.
///
/// A slot whose file the install lacks, or whose file this reader cannot
/// parse, is left out — and a slot left out of an entry that is here at
/// all is the packet's own missing slot, which `Guy::set_anim` gives the
/// three-frame fallback ([`sim::units::anim::MISSING`]). Returns an empty map
/// when either XML is unreadable.
pub fn piece_lengths(install: &Install, graphs: &[String]) -> PieceLengths {
    let apath = install.data("anim_graphics.xml");
    let upath = install.data("unit_graphics.xml");
    let (Ok(atext), Ok(utext)) = (crate::read(&apath), crate::read(&upath)) else {
        return PieceLengths::new();
    };
    let (Ok(adoc), Ok(udoc)) = (crate::parse(&apath, &atext), crate::parse(&upath, &utext)) else {
        return PieceLengths::new();
    };
    let files = anim_files(&adoc);
    let units = unit_anims(&udoc);
    let by_graph = graph_types(graphs);
    let mut frames: BTreeMap<&str, u32> = BTreeMap::new();
    let mut out = PieceLengths::new();
    for (name, rows) in &units {
        let Some(pieces) = entry_pieces(name, &by_graph) else {
            continue;
        };
        let mut lengths: BTreeMap<i8, u32> = BTreeMap::new();
        for (slot, anim) in rows {
            let Some((file, looping)) = files.get(&anim_key(anim)) else {
                continue;
            };
            let n = match frames.get(file.as_str()) {
                Some(&n) => n,
                None => {
                    let Some(n) = file_frames(install.root(), file, *looping) else {
                        continue;
                    };
                    frames.insert(file, n);
                    n
                }
            };
            lengths.insert(*slot, n);
        }
        for &piece in &pieces {
            out.insert(piece, lengths.clone());
        }
    }
    out
}

/// `gpiece → (attack slot → the frames its arrows leave on)`.
///
/// **This is gameplay data, not art.** `CLAUDE.md`'s load-bearing rule is
/// that the *sim crate* depends on no graphics, windowing or async
/// runtime — not that the simulation may never read a table whose
/// filename says "graphics". `unit_graphics.xml`'s `<RELEASEEVENT>` rows
/// are what the original's own simulation consults to decide **when an
/// arrow comes into existence**, and nothing about reading them needs a
/// pixel: the numbers are resolved here, at load, into integer frame
/// counts, and the sim is handed a table.
///
/// `docs/COMBAT.md` §9.0. A unit does not launch its shot from
/// `Unit::fight`: `fight` sets the swing and the reload, and the arrow is
/// added by `GraphicEvents::execute_game_events@008e48e0`, which walks the
/// guy's current animation's event list and fires every `type 1` event the
/// clock has just crossed.
///
/// **Each entry is `(frame, node, harmless)`** (item 853, `docs/ORDERS.md`
/// §39.5): every `<RELEASEEVENT>` is its own `GraphicEvent`, and the walk
/// fires each one it crosses, so two events on one frame are two rounds —
/// the Fighter's two guns. The node is the event's `+0x23`, which
/// `execute_game_events` hands to `get_position` for the launch point and
/// to `Ammo::init` in the package's `angle`. `harmless` is the event's
/// ammo's `do_damage="0"` (`effects_graphics.xml`), which
/// `GraphicPieces::init_ammo_piece_ranges@008f6140` keeps as
/// `ammo_flags & 0x80` and `Ammo::init` turns into the round's flag `0x10`:
/// the round flies, and lands without `Ammo::do_damage`.
///
/// **And a fourth, `missile`** (item 1625, `docs/AI.md` §174): the
/// ammo's `missile="1"`, which the same walk keeps as `ammo_flags & 8`
/// and `Ammo::init@0067bbf0` reads into its `local_30` (`0067c2a5`): a
/// missile round flies a spline, is not led, and never rolls.
///
/// **And a fifth, `flame`** (item 3580, `docs/AI.md` §313): the ammo's
/// `flame="1"`, `ammo_flags & 0x10`, which `Object::do_damage@0064a480`
/// reads on the round's blow to kill with `dtype` 4.
pub type PieceReleases = BTreeMap<i32, BTreeMap<i8, Vec<sim::units::anim::Release>>>;

/// The `<AMMO>` names of `effects_graphics.xml` whose `do_damage` is 0.
///
/// `init_ammo_piece_ranges` reads the attribute with a default of **1**
/// and sets `ammo_flags |= 0x80` when it reads zero; the name is what a
/// `<RELEASEEVENT type=>` is matched against (`init_unit_events`' walk of
/// `ammo_names`, `String::operator==`). The install names three:
/// `NoDamage Tracer`, `NoDamage ArcherArrow` and `ThrowingDagger`.
pub fn harmless_ammo(install: &Install) -> std::collections::BTreeSet<String> {
    ammo_where(install, "do_damage", 0)
}

/// The `<AMMO>` names of `effects_graphics.xml` whose `missile` is 1:
/// `init_ammo_piece_ranges`' `ammo_flags |= 8` (`docs/COMBAT.md` §55.1,
/// `docs/AI.md` §174). The install names ten — the rockets, the torpedo, the SAM's,
/// the Stinger and the four cruise missiles; `CruiseMissile`, the missile
/// *unit*'s own graphic, carries `missile="0"`.
pub fn missile_ammo(install: &Install) -> std::collections::BTreeSet<String> {
    ammo_where(install, "missile", 1)
}

/// The `<AMMO>` names of `effects_graphics.xml` whose `flame` is 1:
/// `init_ammo_piece_ranges`' `ammo_flags |= 0x10` — the fifth attribute it
/// reads, `int_str_array + 0x12994`, which is `internal_strings.xml`'s
/// 3809, `flame` (the other seven land on `puncture`, `puff`, `explosive`,
/// `missile`, `pulsewave`, `flak` and `do_damage`, their own bits). The
/// install names five: `InvisibleFlameAmmo` (the Flamethrower's),
/// `FlamingArrowBolt`, `FlamingArrowBoltLoad`, `FlamingArrow` and
/// `FirelanceAmmo`.
pub fn flame_ammo(install: &Install) -> std::collections::BTreeSet<String> {
    ammo_where(install, "flame", 1)
}

/// The `<AMMO>` names of `effects_graphics.xml` whose `puncture` is 1:
/// `init_ammo_piece_ranges`' `ammo_flags |= 1`, the first attribute it
/// reads (`int_str_array + 0x12958`, `internal_strings.xml`'s `puncture`).
/// The install names six: `BuildingArrow`, `ArcherArrow`,
/// `ArcherArrowWOtip`, `FlamingArrowBolt`, `FlamingArrow` and `Spear`
/// (item 3679, `docs/COMBAT.md` §46.7).
pub fn puncture_ammo(install: &Install) -> std::collections::BTreeSet<String> {
    ammo_where(install, "puncture", 1)
}

/// **The ammo pieces by name** (item 4175, `docs/RENDERER.md` §3.13):
/// each `<AMMO>` of `effects_graphics.xml`, in file order, is the piece
/// [`sim::combat::FIRST_AMMO_PIECE`] plus its index —
/// `GraphicPieces::init_ammo_piece_ranges@008f6140` appends the `<AMMO>`
/// names to `ammo_names` first, one piece each, before the entrenchments,
/// scaffolds, debris and markers. A name is found as
/// `GraphicEvents::init_unit_events@008e2520:280` finds it: the first
/// entry `String::operator==` matches, which compares without case
/// (`CANNONBALL` is the `CannonBall` at 17). Keyed by the lowercase name.
/// `BuildingTraceBullet` is listed twice; the first wins.
pub fn ammo_pieces(install: &Install) -> BTreeMap<String, i32> {
    let mut out = BTreeMap::new();
    for (i, n) in ammo_elements(install).iter().enumerate() {
        out.entry(n.name.to_lowercase())
            .or_insert(sim::combat::FIRST_AMMO_PIECE + i as i32);
    }
    out
}

/// One `<AMMO>` of `effects_graphics.xml`, in file order: its name and
/// the two files `GraphicPieces::init_ammo_data@008f1490` loads for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmmoArt {
    pub name: String,
    pub model: Option<String>,
    pub texture: Option<String>,
}

/// Every `<AMMO>`, in file order: piece `FIRST_AMMO_PIECE + i` is entry `i`.
pub fn ammo_elements(install: &Install) -> Vec<AmmoArt> {
    let path = install.data("effects_graphics.xml");
    let Ok(text) = crate::read(&path) else {
        return Vec::new();
    };
    let Ok(doc) = crate::parse(&path, &text) else {
        return Vec::new();
    };
    let attr = |n: &roxmltree::Node<'_, '_>, k: &str| {
        n.attribute(k)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    doc.descendants()
        .filter(|n| n.has_tag_name("AMMO"))
        .map(|n| AmmoArt {
            name: n.attribute("name").unwrap_or_default().to_string(),
            model: attr(&n, "model"),
            texture: attr(&n, "texture"),
        })
        .collect()
}

/// **`fire_proj` for the types that name an `<INSTANTHIT ammo>`** (item
/// 4175): `GraphicEvents::init_unit_events@008e2520:438`–`447` walks
/// `ammo_names` for the element's `ammo` and writes the type's `UnitType
/// +0x2cc` as `first_ammo_piece` plus the index. The install names four:
/// `CruiseMissile`, `NukeICBM`, `Nuke` and `V2`, on their missiles'
/// `-DEFAULT-AGE0` entries. `TypeIndex → piece`.
pub fn instant_hits(install: &Install, graphs: &[String]) -> BTreeMap<i32, i32> {
    let upath = install.data("unit_graphics.xml");
    let Ok(utext) = crate::read(&upath) else {
        return BTreeMap::new();
    };
    let Ok(udoc) = crate::parse(&upath, &utext) else {
        return BTreeMap::new();
    };
    let by_graph = graph_types(graphs);
    let pieces = ammo_pieces(install);
    let mut out = BTreeMap::new();
    for u in udoc.descendants().filter(|n| n.has_tag_name("UNIT")) {
        let Some(hit) = u
            .children()
            .find(|n| n.has_tag_name("INSTANTHIT"))
            .and_then(|n| n.attribute("ammo"))
        else {
            continue;
        };
        let piece = pieces
            .get(&hit.to_lowercase())
            .copied()
            .unwrap_or(sim::combat::FIRST_AMMO_PIECE);
        let Some(p) = u.attribute("name").and_then(|n| PieceName::parse(n.trim())) else {
            continue;
        };
        for &ty in by_graph
            .get(&p.graph.to_ascii_uppercase())
            .into_iter()
            .flatten()
        {
            out.insert(ty, piece);
        }
    }
    out
}

/// The `<AMMO>` names whose integer attribute `attr` reads `value`.
fn ammo_where(install: &Install, attr: &str, value: i32) -> std::collections::BTreeSet<String> {
    let path = install.data("effects_graphics.xml");
    let Ok(text) = crate::read(&path) else {
        return Default::default();
    };
    let Ok(doc) = crate::parse(&path, &text) else {
        return Default::default();
    };
    doc.descendants()
        .filter(|n| n.has_tag_name("AMMO"))
        .filter(|n| {
            n.attribute(attr)
                .and_then(|v| v.trim().parse::<i32>().ok())
                .is_some_and(|v| v == value)
        })
        .filter_map(|n| n.attribute("name").map(str::to_string))
        .collect()
}

/// `TypeIndex → (node → (minangle, maxangle))`: `unit_graphics.xml`'s
/// `<RESTRICTION>` rows, `GraphicPieces::pivot_restrictions` as
/// `GraphicPieces::init_pivot_restrictions@008ebfe0` builds it.
pub type PivotRestrictions = BTreeMap<i32, BTreeMap<i32, (i32, i32)>>;

/// **Which unit types aim a pivot instead of turning** — the
/// `<RESTRICTION name node minangle maxangle>` rows at the foot of
/// `unit_graphics.xml` (81 of them; `docs/COMBAT.md` §52).
///
/// `GraphicPieces::init_pivot_restrictions@008ebfe0` walks every unit
/// type, names it with `set_unit_graph_name` — the record's `GRAPH` column
/// — and files each row whose `name` is that string under the type. The
/// comparison is `String::operator==@00a1f140`, which is `_wcsicmp`: case
/// does not matter, so `Cruiser` and `Mameluke` bind to `CRUISER` and
/// `MAMELUKE`. The angles are read as floats and handed back truncated by
/// `GraphicPieces::get_restrictions@0090b680`; every shipped one is a
/// whole number of degrees. A type absent from the map has no pivot and
/// turns to face what it shoots.
pub fn pivot_restrictions(install: &Install, graphs: &[String]) -> PivotRestrictions {
    let upath = install.data("unit_graphics.xml");
    let Ok(utext) = crate::read(&upath) else {
        return PivotRestrictions::new();
    };
    let Ok(udoc) = crate::parse(&upath, &utext) else {
        return PivotRestrictions::new();
    };
    let mut out = PivotRestrictions::new();
    for r in udoc.descendants().filter(|n| n.has_tag_name("RESTRICTION")) {
        let (Some(name), Some(node), Some(lo), Some(hi)) = (
            r.attribute("name"),
            r.attribute("node")
                .and_then(|v| v.trim().parse::<i32>().ok()),
            r.attribute("minangle")
                .and_then(|v| v.trim().parse::<i32>().ok()),
            r.attribute("maxangle")
                .and_then(|v| v.trim().parse::<i32>().ok()),
        ) else {
            continue;
        };
        for (i, g) in graphs.iter().enumerate() {
            let ty = 0x32 + i as i32;
            if ty >= 0x192 {
                break;
            }
            if g.trim().eq_ignore_ascii_case(name.trim()) {
                out.entry(ty).or_default().insert(node, (lo, hi));
            }
        }
    }
    out
}

/// A `<RELEASEEVENT starttime=>`'s millisecond stamp as the **game frame**
/// the event list holds it at: `starttime / 67`, truncated, and never
/// below 1.
///
/// **From the listing** (item 542, `docs/COMBAT.md` §50.1). The event list
/// `execute_game_events` walks is built by
/// `GraphicEvents::init_unit_events@008e2520`, not by
/// `GraphicPieces::init_unit_events`, whose copy keeps the milliseconds for
/// the renderer. For each `releaseevent` it writes `event_type` 1, reads
/// `starttime` into `GraphicEvent +0xc`, and at `008e296d`–`008e299f`
/// divides it by 67 (`imul 0x7a44c6b`, `sar edx, 1`, the sign fix) and
/// raises a zero to 1, writing the result to both `start_time` and
/// `end_time`.
///
/// ~~`ms × 3 / 200`, truncated~~, which is what this read until item 542,
/// and it agreed with the original's frame on every release run53, run109
/// and run112 had measured: all six of the Longbowman's, the Bowmen's 666,
/// 833 and 1066, and the Slingers' 1465 and 1532 give the same frame
/// both ways. Only 356 of the install's 2,593 events separate the two
/// readings, and the first a capture reached is the Trireme's `400`: 5
/// by the listing and 6 by the old reading. run127 settles it. The three
/// rounds of every trireme volley launch on frames 5, 9 and 17 of the
/// swing (`400`, `666`, `1200`), four and twelve frames after the first,
/// on both ships and every volley from 622 to 720. The old reading's
/// 6, 9 and 18 would put them three and twelve apart.
///
/// No float: an integer divide at the original's own scale.
pub const fn release_frame(ms: u32) -> u32 {
    let f = ms / 67;
    if f == 0 { 1 } else { f }
}

/// Every player unit piece whose `<UNIT>` names a `skeleton` — the
/// corpse test `DeathObj::init@008d4c60` keeps at `+0x44`
/// (`RData::unit_data +0x20`, `UnitRDataStruct::skeleton`, `-1` when the
/// attribute is absent), which picks the death object's fade
/// (`docs/GOLDEN.md` §127). `graphs` is the `GRAPH` column
/// [`piece_lengths`] takes.
pub fn piece_skeletons(install: &Install, graphs: &[String]) -> std::collections::BTreeSet<i32> {
    let mut out = std::collections::BTreeSet::new();
    let upath = install.data("unit_graphics.xml");
    let Ok(utext) = crate::read(&upath) else {
        return out;
    };
    let Ok(udoc) = crate::parse(&upath, &utext) else {
        return out;
    };
    let by_graph = graph_types(graphs);
    for u in udoc.descendants().filter(|n| n.has_tag_name("UNIT")) {
        let Some(name) = u.attribute("name") else {
            continue;
        };
        if u.attribute("skeleton").is_none() {
            continue;
        }
        let Some(pieces) = entry_pieces(name.trim(), &by_graph) else {
            continue;
        };
        for &piece in &pieces {
            out.insert(piece);
        }
    }
    out
}

/// Every player unit piece's [`PieceReleases`] entry, read from the
/// install.
///
/// `graphs` is the same `GRAPH` column [`piece_lengths`] takes and the
/// name walk is [`piece_tracks`]'s. Only the rows whose `anim` names a
/// slot this crate knows are kept, and each is `(frame, node, harmless)`
/// **in the file's order**, which is the order `init_unit_events` appends
/// them and `execute_game_events` walks them — so two rounds on one frame
/// take their pool slots in that order. **Every event is kept**, a
/// repeated `(frame, node)` too: the original creates a round for each.
///
/// ~~The frames of one slot came back sorted and deduplicated, and the
/// node was dropped~~ (item 853): `FIGHTER`'s sixteen events are eight
/// frames on node 0 and the same eight on node 1, and this crate fired
/// one gun. The 54 pairs repeated at one node (the armoured cars', the
/// machine guns', the infantry's) each carry a `do_damage="0"` round,
/// which is `harmless` here ([`harmless_ammo`]).
///
/// A piece with no `<RELEASEEVENT>` at all is absent, which is the answer
/// for every melee type: `docs/COMBAT.md` §9.0's SEAM — this crate has no
/// reading of `UnitType +0x2cc`, the one other route into
/// `Object::fire_ammo` for a unit, and no capture on this disk reaches it.
pub fn piece_releases(install: &Install, graphs: &[String]) -> PieceReleases {
    let upath = install.data("unit_graphics.xml");
    let Ok(utext) = crate::read(&upath) else {
        return PieceReleases::new();
    };
    let Ok(udoc) = crate::parse(&upath, &utext) else {
        return PieceReleases::new();
    };
    let by_graph = graph_types(graphs);
    let harmless = harmless_ammo(install);
    let missile = missile_ammo(install);
    let flame = flame_ammo(install);
    let puncture = puncture_ammo(install);
    let ammo = ammo_pieces(install);
    let mut out = PieceReleases::new();
    for u in udoc.descendants().filter(|n| n.has_tag_name("UNIT")) {
        let Some(name) = u.attribute("name") else {
            continue;
        };
        let Some(pieces) = entry_pieces(name.trim(), &by_graph) else {
            continue;
        };
        let mut rows: BTreeMap<i8, Vec<sim::units::anim::Release>> = BTreeMap::new();
        for e in u.children().filter(|n| n.has_tag_name("RELEASEEVENT")) {
            let (Some(anim), Some(start)) = (e.attribute("anim"), e.attribute("starttime")) else {
                continue;
            };
            let Some(&(_, slot)) = SLOTS.iter().find(|(s, _)| *s == anim.trim()) else {
                continue;
            };
            let Ok(ms) = start.trim().parse::<u32>() else {
                continue;
            };
            // `get_attrib_num(node, -1)`, kept as the event's `char`.
            let node = e
                .attribute("node")
                .and_then(|v| v.trim().parse::<i8>().ok())
                .unwrap_or(-1);
            let quiet = e.attribute("type").is_some_and(|t| harmless.contains(t));
            let flies = e.attribute("type").is_some_and(|t| missile.contains(t));
            let burns = e.attribute("type").is_some_and(|t| flame.contains(t));
            let sticks = e.attribute("type").is_some_and(|t| puncture.contains(t));
            // `init_unit_events:284`, and `:355`'s `first_ammo_piece` for
            // a name no entry matches.
            let piece = e
                .attribute("type")
                .and_then(|t| ammo.get(&t.to_lowercase()))
                .copied()
                .unwrap_or(sim::combat::FIRST_AMMO_PIECE);
            rows.entry(slot).or_default().push((
                release_frame(ms),
                node,
                quiet,
                flies,
                burns,
                sticks,
                piece,
            ));
        }
        if rows.is_empty() {
            continue;
        }
        for &piece in &pieces {
            out.insert(piece, rows.clone());
        }
    }
    out
}

/// `GRAPH → (get_game_frames(8), get_game_frames(0xc), slot 0xc's
/// releases)` for an anti-air building whose piece is a unit's.
pub type WallPackets = BTreeMap<String, (u32, u32, Vec<(u32, i8, bool, i32)>)>;

/// **The packet an anti-air building's `Wall::inc_time` reads** (item
/// 1112, `docs/COMBAT.md` §84.2), for each `GRAPH` given.
///
/// `GraphicPieces::verify_load@00906550` loads a build piece of type
/// `AIRDEFENSE`, `RADAR` or `SAM` (`0x20b`..`0x20d`) in the first `0x81`
/// build pieces through `init_unit_data` rather than `init_build_data`:
/// the piece is the `<UNIT>` of `unit_graphics.xml` named for the type's
/// `GRAPH`, and its packet has the unit's slots. The name is read here
/// as `GRAPH-DEFAULT-AGE0`, the only `<UNIT>` each of the three has; the
/// string `verify_load` appends (`int_str_array +0x122f0`) is not read.
///
/// A slot the entry does not name is [`sim::units::anim::MISSING`] frames, as
/// `get_game_frames` answers for it; a `GRAPH` with no such `<UNIT>` is
/// absent. The releases are [`piece_releases`]' rows for slot `0xc`
/// (`CHAR_ATTACK2`), in file order.
pub fn wall_packets(install: &Install, graphs: &[String]) -> WallPackets {
    let apath = install.data("anim_graphics.xml");
    let upath = install.data("unit_graphics.xml");
    let (Ok(atext), Ok(utext)) = (crate::read(&apath), crate::read(&upath)) else {
        return WallPackets::new();
    };
    let (Ok(adoc), Ok(udoc)) = (crate::parse(&apath, &atext), crate::parse(&upath, &utext)) else {
        return WallPackets::new();
    };
    let files = anim_files(&adoc);
    let units = unit_anims(&udoc);
    let harmless = harmless_ammo(install);
    let pieces = ammo_pieces(install);
    let length = |rows: &Vec<(i8, String)>, slot: i8| -> u32 {
        rows.iter()
            .find(|(s, _)| *s == slot)
            .and_then(|(_, anim)| files.get(&anim_key(anim)))
            .and_then(|(file, looping)| file_frames(install.root(), file, *looping))
            .unwrap_or(sim::units::anim::MISSING)
    };
    let mut out = WallPackets::new();
    for graph in graphs {
        let name = format!("{}-DEFAULT-AGE0", graph.trim());
        let Some(rows) = units.get(&name) else {
            continue;
        };
        let mut releases = Vec::new();
        if let Some(u) = udoc
            .descendants()
            .find(|n| n.has_tag_name("UNIT") && n.attribute("name").map(str::trim) == Some(&name))
        {
            for e in u.children().filter(|n| n.has_tag_name("RELEASEEVENT")) {
                if e.attribute("anim").map(str::trim) != Some("CHAR_ATTACK2") {
                    continue;
                }
                let Some(ms) = e
                    .attribute("starttime")
                    .and_then(|v| v.trim().parse::<u32>().ok())
                else {
                    continue;
                };
                let node = e
                    .attribute("node")
                    .and_then(|v| v.trim().parse::<i8>().ok())
                    .unwrap_or(-1);
                let quiet = e.attribute("type").is_some_and(|t| harmless.contains(t));
                let piece = e
                    .attribute("type")
                    .and_then(|t| pieces.get(&t.to_lowercase()))
                    .copied()
                    .unwrap_or(sim::combat::FIRST_AMMO_PIECE);
                releases.push((release_frame(ms), node, quiet, piece));
            }
        }
        out.insert(
            graph.trim().to_string(),
            (
                length(rows, sim::units::anim::WALK),
                length(rows, sim::units::anim::ATTACK2),
                releases,
            ),
        );
    }
    out
}

/// `guy_scale`, the executable's own `float` at `00c06244` — `Guy.obj`'s
/// only exported datum in `rise_z.map`, initialised in `.data` to
/// **4.8** and written nowhere but three `ConsoleWin::run_cmd` arms.
///
/// It is a *drawing* scale everywhere else, and it would not be here at
/// all except that `Guy::update_gpiece@005d8530:61` multiplies the crew
/// guy's follow offset by it before truncating to an integer. So the
/// number a follower stands at is 4.8 times what the graphics file says,
/// and the fraction is what makes the truncation matter.
const GUY_SCALE: f32 = 4.8;

/// `gpiece → (track_dx, track_dy)`: the offset a **crew** guy stands at
/// behind and beside the guy it follows, in position units.
///
/// `Guy::update_gpiece@005d8530` computes it for a guy whose `guy_num` is
/// non-zero, from the piece's `RData`:
///
/// ```text
/// track_dx = (int)(track_offsetx * guy_scale * scale)
/// track_dy = (int)(guy_scale * track_offsety * scale)
/// ```
///
/// — `RData::scale` (`+0x88`) is the `<UNIT scale=>` attribute and
/// `UnitRDataStruct::track_offsetx / track_offsety` (`+0xc` / `+0x10`) are
/// `trackoffsetx` / `trackoffsety`. A guy whose piece names neither stands
/// on its leader, and both branches of `Guy::move` that read the pair test
/// it against zero, so an absent attribute is a real answer rather than a
/// gap: `0` means "no track", and the guy is carried rather than walked.
///
/// The floats are the install's own decimals and the multiply is done
/// here, once, at load — the sim is handed integers (`CLAUDE.md`, "no
/// floating point in the sim").
pub type PieceTracks = BTreeMap<i32, (i32, i32)>;

/// Every unit piece's [`PieceTracks`] entry, read from the install.
///
/// `graphs` is the same `GRAPH` column [`piece_lengths`] takes, and the
/// name walk is the same: record `i` is `TypeIndex 0x32 + i`, and a
/// `<UNIT>` name that `get_unit_gpiece` can never build is skipped.
///
/// Diff-backed: run56's start dump puts the human scout's dog at
/// `(5790, 7979)` with its man at `(5784, 8088)`, and the pair
/// `(-96, 48)` this reads for piece `13043` reproduces that to the unit
/// through `Guy::set_new_location`'s rotation (`docs/MOVEMENT.md`, "The
/// follower's destination").
pub fn piece_tracks(install: &Install, graphs: &[String]) -> PieceTracks {
    let upath = install.data("unit_graphics.xml");
    let Ok(utext) = crate::read(&upath) else {
        return PieceTracks::new();
    };
    let Ok(udoc) = crate::parse(&upath, &utext) else {
        return PieceTracks::new();
    };
    let by_graph = graph_types(graphs);
    let mut out = PieceTracks::new();
    for u in udoc.descendants().filter(|n| n.has_tag_name("UNIT")) {
        let Some(name) = u.attribute("name") else {
            continue;
        };
        let Some(pieces) = entry_pieces(name.trim(), &by_graph) else {
            continue;
        };
        let attr = |k: &str| -> f32 {
            u.attribute(k)
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0.0)
        };
        // `scale` is the one attribute that defaults to one rather than
        // zero: `RData::scale` is written by the loader for every piece,
        // and a `<UNIT>` without it draws at its model's own size.
        let scale: f32 = u
            .attribute("scale")
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(1.0);
        let (ox, oy) = (attr("trackoffsetx"), attr("trackoffsety"));
        if ox == 0.0 && oy == 0.0 {
            continue;
        }
        // The two multiplies in the original's own order — the x one folds
        // `guy_scale` into the offset first, the y one leads with it — so
        // the roundings land where they land.
        let dx = (ox * GUY_SCALE * scale) as i32;
        let dy = (GUY_SCALE * oy * scale) as i32;
        for &piece in &pieces {
            out.insert(piece, (dx, dy));
        }
    }
    out
}

/// `anim_graphics.xml`'s `<ANIM name= file=>`: the animation's name as a
/// `<UNIT>` cites it, the file it resolves to (`.\art\x.bha` kept as the
/// install-relative path it is), and **whether it loops**.
///
/// The looping flag is the section the row sits in, and it is
/// `GraphicPieces::init_anims_pool@008fca40`'s own reading: four passes,
/// `AnimMgr::add(name, 1)` under `<LOOPING>` and `AnimMgr::add(name, 0)`
/// under `<NONLOOPING>`, `<BUILDING>` and `<PATH>` alike — `AnimMgr::add
/// @0053ac00` writes `loopings[i] = param_2 != 0`. So a section this
/// reader does not know is not looping, which is what the original does
/// with the two it has beyond the pair the name suggests.
///
/// It is not a decoration: [`game_frames`] reads it (§3.1).
///
/// **The key is the name case-folded, and the first row wins**
/// (`docs/ANIM.md` §12). A `<UNIT>`'s `file=` is resolved by
/// `GraphicPieces::decipher_animation@008face0`: a walk of `anim_names` in
/// load order that returns the first match under `_wcsicmp`. So `"Galley
/// Default"` finds `"Galley default"` — 46 of the names `unit_graphics.xml`
/// cites resolve only that way — and a case-sensitive map read each of them
/// as a slot the packet lacks, three frames. Look names up through
/// [`anim_key`].
fn anim_files(doc: &roxmltree::Document<'_>) -> BTreeMap<String, (String, bool)> {
    let mut out = BTreeMap::new();
    for n in doc.descendants().filter(|n| n.has_tag_name("ANIM")) {
        let (Some(name), Some(file)) = (n.attribute("name"), n.attribute("file")) else {
            continue;
        };
        let looping = n.parent().is_some_and(|p| p.has_tag_name("LOOPING"));
        out.entry(anim_key(name))
            .or_insert((file.trim().to_string(), looping));
    }
    out
}

/// An animation name as [`anim_files`] keys it: trimmed and ASCII
/// case-folded, `_wcsicmp`'s comparison on the names the install ships.
fn anim_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

/// `unit_graphics.xml`'s `<UNIT name="…"><ANIM name="CHAR_…" file="…"/>`:
/// each entry's slot rows. Only `<ANIM>` children of a `<UNIT>` count —
/// the `EXISTEVENT`/`SOUNDEVENT` rows name the same slots and are not the
/// packet.
fn unit_anims(doc: &roxmltree::Document<'_>) -> BTreeMap<String, Vec<(i8, String)>> {
    let mut out: BTreeMap<String, Vec<(i8, String)>> = BTreeMap::new();
    for u in doc.descendants().filter(|n| n.has_tag_name("UNIT")) {
        let Some(name) = u.attribute("name") else {
            continue;
        };
        let mut rows = Vec::new();
        for a in u.children().filter(|n| n.has_tag_name("ANIM")) {
            let (Some(slot), Some(file)) = (a.attribute("name"), a.attribute("file")) else {
                continue;
            };
            let Some(&(_, i)) = SLOTS.iter().find(|(s, _)| *s == slot.trim()) else {
                continue;
            };
            rows.push((i, file.trim().to_string()));
        }
        // **An entry with no `<ANIM>` child is still an entry**, and it is
        // the answer for a crew figure: all sixty of the shipped
        // animation-less `<UNIT>`s are `-CREW{k}`, and each of them loads a
        // model and so a piece — an *empty* `AnimationPacket`. Dropping it
        // cost twice over: `get_unit_gpiece`'s existence walk missed the
        // piece and fell to `first_unit_piece`, and every slot then read
        // the wrong art's length instead of the three frames
        // `get_game_frames` returns for a slot no packet names
        // (`docs/ANIM.md` §3.6).
        out.insert(name.trim().to_string(), rows);
    }
    out
}

/// `AnimMgr::force_load`'s `frames[]` for one animation file: the root
/// node's last key time in milliseconds, at fifteen frames a second.
pub fn file_frames(root: &Path, rel: &str, looping: bool) -> Option<u32> {
    let bytes = std::fs::read(resolve(root, rel)?).ok()?;
    Some(game_frames(&key_times(&bytes)?, looping))
}

/// [`resolve`], for the other art readers (`crate::mountains`, the
/// `client` crate's meshes).
pub fn resolve_art(root: &Path, rel: &str) -> Option<PathBuf> {
    resolve(root, rel)
}

/// `.\art\bird_flap.bha` as a path in this install, tolerating the case
/// the XML writes and the case the file carries — `bird_flap.bha` beside
/// `sheep_idle1.BHa`, and the two disagree in the shipped data.
fn resolve(root: &Path, rel: &str) -> Option<PathBuf> {
    let rel = rel.trim_start_matches(".\\").replace('\\', "/");
    let direct = root.join(&rel);
    if direct.is_file() {
        return Some(direct);
    }
    let (dir, file) = rel.rsplit_once('/')?;
    let want = file.to_ascii_lowercase();
    std::fs::read_dir(root.join(dir))
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.to_ascii_lowercase() == want)
        })
}

/// The root `AnimObj` chunk's key times, in milliseconds.
///
/// The file is a chunk stream: `{u32 size; u16 id; u16 version}` headers
/// where `size` is measured from the `size` field itself. The outer chunk
/// is the file, the next is `id 8` (the object), and the one at offset 16
/// is `id 7` — `AnimObj::load_hier`'s node: a `u32` key count, then that
/// many 36-byte keys whose **first float is the key's own duration in
/// seconds**. `load_hier` accumulates `int(seconds · 1000)` into a `u16`
/// per key, which is the array `force_load` reads the last element of.
/// Child nodes follow and carry their own copies; only the root's count.
///
/// The count is the chunk's **own** field and the keys need not fill it:
/// `load_hier` reads `header[1].size` keys from `header + 12` and never
/// compares the two (`0054b700`, the `local_18` loop). `man_walk.bha`
/// says thirty and carries thirty-one, and the strict equality this once
/// had rejected it — and with it the citizen's whole walk
/// (`docs/ANIM.md` §3.2). A chunk whose keys **overrun** it is still not
/// one this reader understands.
pub fn key_times(bytes: &[u8]) -> Option<Vec<u16>> {
    let word = |at: usize| -> Option<u32> {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let size = word(16)? as usize;
    let n = word(24)? as usize;
    if 28 + n.checked_mul(36)? > 16 + size || n == 0 {
        return None;
    }
    let mut acc: i64 = 0;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let secs = f32::from_bits(word(28 + i * 36)?);
        acc += i64::from((secs * 1000.0) as i32 & 0xffff);
        out.push((acc & 0xffff) as u16);
    }
    Some(out)
}

/// `AnimMgr::force_load@0053ade0`'s `frames[]` — the whole of the
/// conversion from the file's milliseconds to the animation clock's
/// frames, and the reason a length is what it is. Fifteen frames a second,
/// rounded half up.
///
/// **A non-looping animation drops its last key.** `force_load` reads the
/// root node's last key time and then, when `loopings[i] == 0` and that
/// time is not zero, replaces it with the **second to last** — writing it
/// back into the node's own array — before the conversion. A looping
/// animation's last key is the frame that returns to the first, so it is
/// part of the cycle; a non-looping one's is the pose it ends on and is
/// not played through.
///
/// One frame, and it is the whole of East Indies' word at 1570:
/// `lumberjack_dump.bha`'s keys end 2157, 2190, so the citizen's
/// `CHAR_DUMP_WOOD` is 32 rather than 33 and its wrap falls a frame
/// earlier. Every `GUY` block in the corpus agrees — see
/// `crate::diff::tests::the_install_s_piece_lengths_match_every_dumped_
/// clock`.
pub fn game_frames(times: &[u16], looping: bool) -> u32 {
    let last = u32::from(*times.last().unwrap_or(&0));
    let ms = match times.len() {
        n if !looping && last != 0 && n >= 2 => u32::from(times[n - 2]),
        _ => last,
    };
    // `(int)(ms · 3 / 200.0f)`, then `+1` when the remainder is at least a
    // half — integer arithmetic for the same answer.
    let n = ms * 3;
    n / 200 + u32::from(n % 200 * 2 >= 200)
}

/// One `<UNIT>` entry's art: the model it loads and the animation each of
/// its slots names (`docs/FORMATS.md` §3, "From a piece to its files").
///
/// `GraphicPieces` loads an entry's `model=` as the piece's `.BH3` and
/// builds its `AnimationPacket` from the `<ANIM>` rows, each name resolved
/// through `anim_graphics.xml` as [`anim_files`] does. An entry is every
/// piece [`entry_pieces`] gives it, so pieces sharing an entry share its
/// files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitArt {
    pub name: String,
    /// `model=`, as the XML writes it (`.\art\minigun_gunner.bh3`).
    pub model: String,
    /// `scale=`, as the XML writes it (`"1.1"`), empty when absent — the
    /// model's size the client draws it at (item 4036).
    pub scale: String,
    /// `texture=`, as the XML writes it (`.\art\assault_infantry.tga`),
    /// empty when absent — the image the model's texcoords sample (item
    /// 4099).
    pub texture: String,
    /// `(slot, the name the row cites, its file if anim_graphics names it)`.
    pub anims: Vec<(i8, String, Option<String>)>,
    /// The graphic pieces the entry is; empty when it is none.
    pub pieces: Vec<i32>,
    /// `shadowtype=`, `shadowwidth=` and `shadowheight=` as the XML
    /// writes them, empty when absent (item 4182): the decal
    /// `GraphicPieces::draw_unit_shadow@008e8fa0` lays under the guy, its
    /// type a name of [`shadow_files`].
    pub shadow: [String; 3],
}

/// `effects_graphics.xml`'s `<SHADOWS>`: each `<SHADOW>`'s `name` and
/// `file`, in the file's order (item 4182) — the list `init_unit_data`
/// matches a `shadowtype` against (`shadow_names`).
pub fn shadow_files(install: &Install) -> Vec<(String, String)> {
    let path = install.data("effects_graphics.xml");
    let Ok(text) = crate::read(&path) else {
        return Vec::new();
    };
    let Ok(doc) = crate::parse(&path, &text) else {
        return Vec::new();
    };
    doc.descendants()
        .filter(|n| n.has_tag_name("SHADOWS"))
        .flat_map(|s| s.children().filter(|n| n.has_tag_name("SHADOW")))
        .filter_map(|n| {
            Some((
                n.attribute("name")?.trim().to_string(),
                n.attribute("file")?.trim().to_string(),
            ))
        })
        .collect()
}

/// Every `<UNIT>` entry's [`UnitArt`], in name order. `graphs` is the
/// `GRAPH` column, as [`piece_lengths`] takes it. Empty when either XML is
/// unreadable.
pub fn unit_art(install: &Install, graphs: &[String]) -> Vec<UnitArt> {
    let apath = install.data("anim_graphics.xml");
    let upath = install.data("unit_graphics.xml");
    let (Ok(atext), Ok(utext)) = (crate::read(&apath), crate::read(&upath)) else {
        return Vec::new();
    };
    let (Ok(adoc), Ok(udoc)) = (crate::parse(&apath, &atext), crate::parse(&upath, &utext)) else {
        return Vec::new();
    };
    let files = anim_files(&adoc);
    let units = unit_anims(&udoc);
    let by_graph = graph_types(graphs);
    let mut out = Vec::new();
    for u in udoc.descendants().filter(|n| n.has_tag_name("UNIT")) {
        let (Some(name), Some(model)) = (u.attribute("name"), u.attribute("model")) else {
            continue;
        };
        let name = name.trim().to_string();
        let anims = units
            .get(&name)
            .into_iter()
            .flatten()
            .map(|(slot, anim)| {
                let file = files.get(&anim_key(anim)).map(|(f, _)| f.clone());
                (*slot, anim.clone(), file)
            })
            .collect();
        let pieces = entry_pieces(&name, &by_graph)
            .or_else(|| bird_pieces(&name, graphs))
            .unwrap_or_default();
        out.push(UnitArt {
            name,
            model: model.trim().to_string(),
            scale: u.attribute("scale").unwrap_or_default().trim().to_string(),
            texture: u
                .attribute("texture")
                .unwrap_or_default()
                .trim()
                .to_string(),
            anims,
            pieces,
            shadow: ["shadowtype", "shadowwidth", "shadowheight"]
                .map(|k| u.attribute(k).unwrap_or_default().trim().to_string()),
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Why one file did not decode, by the path the XML writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArtFailure {
    /// No file in the install answers the path, in any case.
    Missing(String),
    /// The file is there and the decoder names what is wrong with it.
    Undecodable(String, String),
    /// A slot cites an animation `anim_graphics.xml` does not name.
    Unnamed(String),
}

/// The pieces the resolution names, and how many decode whole
/// ([`piece_decoding`]).
#[derive(Clone, Debug, Default)]
pub struct PieceDecoding {
    /// Distinct pieces with an entry: the N.
    pub pieces: usize,
    /// Those whose model and every slot's animation decode: the K.
    pub decoded: usize,
    /// Entries read, and entries that are at least one piece.
    pub entries: usize,
    pub entries_with_pieces: usize,
    /// Pieces whose model's node tree and every played animation's have
    /// the same shape (a count beside K, never part of it).
    pub same_shape: usize,
    /// Every failing file, with the pieces it costs.
    pub failures: BTreeMap<String, (ArtFailure, usize)>,
    /// The K's pieces themselves, so a count over another set — the
    /// renderer's M (item 4020) — can be taken against them.
    pub decoded_pieces: std::collections::BTreeSet<i32>,
    /// Every piece with an entry, by what fails in it: whether its model
    /// decodes, and the slots whose animation does not — so §5.4's
    /// condition 1 can be read on the slots a run plays
    /// ([`PieceDecoding::decodes_on`]).
    pub by_piece: BTreeMap<i32, PieceFiles>,
}

/// One piece's files, as [`piece_decoding`] found them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PieceFiles {
    /// The `.bh3` decodes.
    pub model: bool,
    /// The slots whose `.bha` is missing, unnamed or undecodable.
    pub failing_slots: std::collections::BTreeSet<i8>,
}

impl PieceDecoding {
    /// **`docs/RENDERER.md` §5.4's condition 1, as it is written** (item
    /// 4028): the piece's `.BH3` and every `.BHa` *its captured slots
    /// name* decode — `played` is the slots a run shows the piece in. A
    /// slot the run never plays is not drawn, so its file is not asked;
    /// [`PIECES_DECODING`] stays the stricter every-slot count. `false` for
    /// a piece no entry names.
    pub fn decodes_on(&self, piece: i32, played: &std::collections::BTreeSet<i8>) -> bool {
        self.by_piece
            .get(&piece)
            .is_some_and(|f| f.model && f.failing_slots.is_disjoint(played))
    }
}

/// **Pieces decoding: K of N** (item 4019; `docs/RENDERER.md` §5.4's
/// condition 1). N is the distinct graphic pieces a `<UNIT>` entry is
/// ([`unit_art`]); a piece counts toward K when its `.bh3` and every `.bha`
/// its slots name decode under `render`'s readers. Measured on the user's
/// install by [`piece_decoding`] (`the_pieces_decode`): 1,452 pieces from
/// 1,399 of 1,435 entries, 13 short — three slots name a `.bha` the
/// install does not ship (`armsuplytruck_attackwalk`, `ceo_crew3_idle3`,
/// `siegeship_ship_walk`: 14 pieces' worth, overlapping) and one model,
/// `citizen6_fem_NA.bh3`, has a node run past its vertices. New, so no
/// delta. Item 4028: **(1465, 1478)**, +26 of +26 — the gaia entries
/// (`{GRAPH}-TYPE{v}`, [`bird_pieces`]) are pieces now, 26 entries of one
/// piece each (1,425 of the 1,435 entries), and every one decodes.
pub const PIECES_DECODING: (usize, usize) = (1465, 1478);

/// Of [`PIECES_DECODING`]'s K, the pieces whose model's node tree and every
/// played animation's tree have the same shape — beside K, never part of
/// it; what the other 638 mean is the next render item's question
/// (`docs/FORMATS.md` §3, "What is not established"). Item 4028: 823,
/// +22 of the 26 gaia pieces.
pub const SAME_SHAPE: usize = 823;

/// Resolve every entry's files and decode each once: `K of N`, and every
/// file that costs a piece named with its reason.
pub fn piece_decoding(install: &Install, graphs: &[String]) -> PieceDecoding {
    let mut models: BTreeMap<String, Result<render::bh3::Model, ArtFailure>> = BTreeMap::new();
    let mut anims: BTreeMap<String, Result<render::bha::Anim, ArtFailure>> = BTreeMap::new();
    let read = |rel: &str| -> Result<Vec<u8>, ArtFailure> {
        let path = resolve(install.root(), rel).ok_or_else(|| ArtFailure::Missing(rel.into()))?;
        std::fs::read(path).map_err(|_| ArtFailure::Missing(rel.into()))
    };
    let mut out = PieceDecoding::default();
    let mut seen = std::collections::BTreeSet::new();
    for e in unit_art(install, graphs) {
        out.entries += 1;
        if e.pieces.is_empty() {
            continue;
        }
        out.entries_with_pieces += 1;
        let model = models
            .entry(e.model.clone())
            .or_insert_with(|| {
                let b = read(&e.model)?;
                render::bh3::decode(&b)
                    .map_err(|err| ArtFailure::Undecodable(e.model.clone(), err.to_string()))
            })
            .clone();
        let mut fails: Vec<(String, ArtFailure)> = Vec::new();
        let mut files = PieceFiles {
            model: model.is_ok(),
            ..PieceFiles::default()
        };
        let mut shapes_agree = true;
        match &model {
            Ok(_) => {}
            Err(f) => fails.push((e.model.clone(), f.clone())),
        }
        for (slot, anim, file) in &e.anims {
            let Some(file) = file else {
                files.failing_slots.insert(*slot);
                fails.push((anim.clone(), ArtFailure::Unnamed(anim.clone())));
                continue;
            };
            let a = anims.entry(file.clone()).or_insert_with(|| {
                let b = read(file)?;
                render::bha::decode(&b)
                    .map_err(|err| ArtFailure::Undecodable(file.clone(), err.to_string()))
            });
            match (a, &model) {
                (Ok(a), Ok(m)) => shapes_agree &= a.shape() == m.shape(),
                (Ok(_), Err(_)) => shapes_agree = false,
                (Err(f), _) => {
                    files.failing_slots.insert(*slot);
                    fails.push((file.clone(), f.clone()));
                }
            }
        }
        let fresh: Vec<i32> = e
            .pieces
            .iter()
            .copied()
            .filter(|p| seen.insert(*p))
            .collect();
        for p in &fresh {
            out.by_piece.insert(*p, files.clone());
        }
        if fails.is_empty() {
            out.decoded_pieces.extend(&fresh);
        }
        let fresh = fresh.len();
        out.pieces += fresh;
        if fails.is_empty() {
            out.decoded += fresh;
            out.same_shape += if shapes_agree { fresh } else { 0 };
        }
        for (key, f) in fails {
            out.failures.entry(key).or_insert((f, 0)).1 += fresh;
        }
    }
    out
}

/// `first_build_piece`, `num_build_pieces` and `num_build_pieces_per_age`
/// as `GraphicPieces::init_piece_ranges@008f70e0` writes them: `0xc606`
/// (the unit pieces' end), `0x81` (one per building record — the 129 of
/// `buildingrules.xml`, `TypeIndex` `0x19e + i`) and `0x489`, nine art
/// styles of 129; `total_num_build_pieces` `0x2448` is eight ages of it.
pub const FIRST_BUILD_PIECE: i32 = 0xc606;
pub const NUM_BUILD_PIECES: i32 = 0x81;
pub const BUILD_PIECES_PER_AGE: i32 = 0x489;

/// The nine building art styles, by the index a nation's
/// `BUILD_CONTINENT` carries — `say_build_art_style_name@006f03f0`'s
/// switch: arm 0 is the empty string, which `build_node_from_gpiece`
/// writes as `Default` (string 402), and arms 1–8 are
/// `internal_strings.xml` entries 6920–6927 (`0x21ca0 / 0x14` on): `NEurope`,
/// `Asian`, `Korean`, `American`, `Arab`, `Iroquois`, `Colonial`,
/// `EIndian`. Upper case here, as the loader's comparison ignores case.
pub const BUILD_STYLES: [&str; 9] = [
    "DEFAULT", "NEUROPE", "ASIAN", "KOREAN", "AMERICAN", "ARAB", "IROQUOIS", "COLONIAL", "EINDIAN",
];

/// **A building piece's `<BUILD>` name** (item 4028), as
/// `GraphicPieces::build_node_from_gpiece@008fd550` builds it: the
/// record's `GRAPH`, `-`, the style, `-AGE`, the age — with the piece read
/// back as `first_build_piece + 0x81·style + 0x489·age + (TypeIndex −
/// 0x19e)`, the arithmetic `get_build_gpiece@0090bce0` adds. The age is the
/// raw age, not a bracket (`(piece − first) / 0x489`). `build_graphs` is
/// `buildingrules.xml`'s `GRAPH` column in file order. `(TypeIndex, name)`,
/// or `None` for a piece outside the building range.
pub fn build_piece_name(piece: i32, build_graphs: &[String]) -> Option<(i32, String)> {
    let r = piece - FIRST_BUILD_PIECE;
    if !(0..BUILD_PIECES_PER_AGE * 8).contains(&r) {
        return None;
    }
    let (age, style, ty) = (
        r / BUILD_PIECES_PER_AGE,
        (r % BUILD_PIECES_PER_AGE) / NUM_BUILD_PIECES,
        r % NUM_BUILD_PIECES,
    );
    let graph = build_graphs.get(ty as usize)?.trim().to_ascii_uppercase();
    Some((
        0x19e + ty,
        format!("{graph}-{}-AGE{age}", BUILD_STYLES[style as usize]),
    ))
}

/// What one building piece's `<BUILD>` names, as
/// `GraphicPieces::init_build_data@008f2f20` and `init_build_events@008ea3c0`
/// read it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildArt {
    pub piece: i32,
    pub type_index: i32,
    /// The `<BUILD name>` the piece is.
    pub name: String,
    /// `texture=` — the building's sprite. A texture is outside §5.4's
    /// condition 1, as a unit's is.
    pub texture: String,
    /// `spritexoff=`, `spriteyoff=` (strings 3852 and 3853, each `-1` when
    /// absent): the sprite's anchor in its own pixels, which
    /// `init_build_data` writes into the sprite packet (`packets[0xf]`) and
    /// `SpriteMethod::execute@0091ca30` subtracts, scaled, from the
    /// building's screen point (item 4076).
    pub sprite_offset: (i32, i32),
    /// `zbuffer=`, the depth model: `init_build_data` loads the piece only
    /// when it is empty or the file exists.
    pub zbuffer: Option<String>,
    /// `anim=`, through `decipher_animation`: `(name, file)`.
    pub anim: Option<(String, Option<String>)>,
    /// Each distinct `EXISTEVENT`'s `subject` — an ammo piece, matched to
    /// `effects_graphics.xml`'s `<AMMO name>` without case — with its
    /// `model=`, and its `anim` with its file.
    pub addons: Vec<(String, Option<String>, String, Option<String>)>,
}

/// **The building pieces' art, resolved through the loader** (item 4028):
/// each piece's [`build_piece_name`], the `<BUILD>` of that name under
/// `building_graphics.xml`'s `<BUILDINGS>` (compared without case,
/// `build_node_from_gpiece@00905050`), and what it names. A piece with no
/// `<BUILD>` maps to `Err` of the name it built — the loader's
/// `LIBERR_FILE_NOT_FOUND` (`init_build_data@008f2f20`); a piece outside
/// the building range, to `Err` of the empty name.
pub fn build_art(
    install: &Install,
    pieces: &std::collections::BTreeSet<i32>,
) -> BTreeMap<i32, Result<BuildArt, String>> {
    let mut out = BTreeMap::new();
    let (bpath, epath, apath) = (
        install.data("building_graphics.xml"),
        install.data("effects_graphics.xml"),
        install.data("anim_graphics.xml"),
    );
    let (Ok(btext), Ok(etext), Ok(atext)) = (
        crate::read(&bpath),
        crate::read(&epath),
        crate::read(&apath),
    ) else {
        return out;
    };
    let (Ok(bdoc), Ok(edoc), Ok(adoc)) = (
        crate::parse(&bpath, &btext),
        crate::parse(&epath, &etext),
        crate::parse(&apath, &atext),
    ) else {
        return out;
    };
    let Ok(rules) = install.buildings() else {
        return out;
    };
    let graphs: Vec<String> = rules
        .records
        .iter()
        .map(|r| r.text("GRAPH").unwrap_or_default().to_string())
        .collect();
    let files = anim_files(&adoc);
    let ammo: BTreeMap<String, Option<String>> = edoc
        .descendants()
        .filter(|n| n.has_tag_name("AMMO"))
        .filter_map(|n| {
            let name = n.attribute("name")?.trim().to_ascii_uppercase();
            Some((name, n.attribute("model").map(|m| m.trim().to_string())))
        })
        .collect();
    let builds: BTreeMap<String, roxmltree::Node<'_, '_>> = bdoc
        .descendants()
        .filter(|n| n.has_tag_name("BUILD"))
        .filter_map(|n| Some((n.attribute("name")?.trim().to_ascii_uppercase(), n)))
        .collect();
    let named = |n: roxmltree::Node<'_, '_>, a: &str| {
        n.attribute(a)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    };
    for &piece in pieces {
        let Some((type_index, name)) = build_piece_name(piece, &graphs) else {
            out.insert(piece, Err(String::new()));
            continue;
        };
        let Some(&b) = builds.get(&name) else {
            out.insert(piece, Err(name));
            continue;
        };
        let anim = named(b, "anim").map(|a| {
            let f = files.get(&anim_key(&a)).map(|(f, _)| f.clone());
            (a, f)
        });
        let mut addons: Vec<(String, Option<String>, String, Option<String>)> = b
            .children()
            .filter(|n| n.has_tag_name("EXISTEVENT"))
            .filter_map(|n| {
                let subject = named(n, "subject")?;
                let a = named(n, "anim").unwrap_or_default();
                let model = ammo.get(&subject.to_ascii_uppercase()).cloned().flatten();
                let f = files.get(&anim_key(&a)).map(|(f, _)| f.clone());
                Some((subject, model, a, f))
            })
            .collect();
        addons.sort();
        addons.dedup();
        let num = |a: &str| {
            named(b, a)
                .and_then(|v| v.parse::<i32>().ok())
                .unwrap_or(-1)
        };
        out.insert(
            piece,
            Ok(BuildArt {
                piece,
                type_index,
                name,
                texture: named(b, "texture").unwrap_or_default(),
                sprite_offset: (num("spritexoff"), num("spriteyoff")),
                zbuffer: named(b, "zbuffer"),
                anim,
                addons,
            }),
        );
    }
    out
}

/// **The building pieces' look, as the loader holds it** (item 4035): which
/// `<BUILD>` entries load, and each type's `halfoffset`. A piece loads —
/// `data_pieces[piece] != 0` after `verify_load` — when its
/// [`build_piece_name`] names a `<BUILD>` whose `zbuffer` is empty or a file
/// of the install (`init_build_data@008f2f20`). `halfoffset` is
/// `internal_strings.xml`'s `halfoffset`, the attribute `init_build_data`
/// reads into `BuildRDataStruct::build_half_tcoord`, default 0; every
/// `<BUILD>` of one `GRAPH` carries the same value, so it is kept by type.
///
/// Item 4061 adds what the foundation and the scaffold read: each piece's
/// `constructsize` (`BuildRDataStruct +0x28`, `build_construction`, which
/// unlike `halfoffset` differs between one `GRAPH`'s ages — the Asian,
/// Arab and East Indian cities), the install's `<SCAFFOLD_TIMING>` tables
/// ([`scaffold_timing`]), and, once [`BuildLook::with_types`] has run,
/// each type's footprint, domain and lineage.
#[derive(Clone, Debug, Default)]
pub struct BuildLook {
    /// The pieces that load, by piece number.
    loads: std::collections::BTreeSet<i32>,
    /// `halfoffset` by `TypeIndex`.
    half: BTreeMap<i32, i32>,
    /// `constructsize` by piece number, where it is not 0.
    construct: BTreeMap<i32, i32>,
    /// What the two construction passes read of a type, by `TypeIndex`.
    kinds: BTreeMap<i32, ConstructKind>,
    /// The install's `<SCAFFOLD_TIMING>` tables.
    timing: render::draw::ScaffoldTiming,
}

/// What `WallOut::draw_underlays@006410a0`, `GraphicPieces::
/// get_construct_gpiece@00907840` and `GraphicPieces::draw_scaffold
/// @00907420` read of a building type (item 4061).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ConstructKind {
    /// `BuildTypeData +0x234` and `+0x238`: the footprint.
    x_size: i32,
    y_size: i32,
    /// `BuildTypeData +0x218 == 1`: the water domain.
    water: bool,
    /// `is(0x1b6)`: of the Senate's lineage.
    senate: bool,
    /// `is(0x1a1)`: the farm, which draws neither.
    farm: bool,
}

impl BuildLook {
    pub fn new(install: &Install) -> Self {
        let path = install.data("building_graphics.xml");
        let (Ok(text), Ok(rules)) = (crate::read(&path), install.buildings()) else {
            return Self::default();
        };
        let Ok(doc) = crate::parse(&path, &text) else {
            return Self::default();
        };
        let graphs: Vec<String> = rules
            .records
            .iter()
            .map(|r| r.text("GRAPH").unwrap_or_default().to_string())
            .collect();
        let builds: BTreeMap<String, roxmltree::Node<'_, '_>> = doc
            .descendants()
            .filter(|n| n.has_tag_name("BUILD"))
            .filter_map(|n| Some((n.attribute("name")?.trim().to_ascii_uppercase(), n)))
            .collect();
        let mut look = Self::default();
        for piece in FIRST_BUILD_PIECE..FIRST_BUILD_PIECE + 8 * BUILD_PIECES_PER_AGE {
            let Some((type_index, name)) = build_piece_name(piece, &graphs) else {
                continue;
            };
            let Some(b) = builds.get(&name) else {
                continue;
            };
            let zbuffer = b.attribute("zbuffer").map(str::trim).unwrap_or_default();
            if zbuffer.is_empty() || resolve(install.root(), zbuffer).is_some() {
                look.loads.insert(piece);
            }
            let int = |key: &str| -> i32 {
                b.attribute(key)
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(0)
            };
            look.half.entry(type_index).or_insert(int("halfoffset"));
            let construct = int("constructsize");
            if construct != 0 {
                look.construct.insert(piece, construct);
            }
        }
        look.timing = scaffold_timing(install).unwrap_or_default();
        look
    }

    /// The look with each type's footprint, domain and lineage, read from
    /// `types` — the loaded `BUILDING` rows in the tree's order, the
    /// `TypeIndex` [`crate::load::BASE_BUILDTYPES`] on. Without it, no
    /// building has a foundation or a scaffold.
    pub fn with_types(mut self, types: &[sim::cities::build::BuildType]) -> Self {
        use sim::cities::build::{BuildDomain, Ident, is};
        self.kinds = (0..types.len())
            .map(|t| {
                let ty = &types[t];
                let kind = ConstructKind {
                    x_size: ty.x_size,
                    y_size: ty.y_size,
                    water: ty.domain() == BuildDomain::Water,
                    senate: is(types, t, Ident::Senate),
                    farm: is(types, t, Ident::Farm),
                };
                (crate::load::BASE_BUILDTYPES + t as i32, kind)
            })
            .collect();
        self
    }

    /// The install's `<SCAFFOLD_TIMING>` tables ([`scaffold_timing`]).
    pub fn scaffold_timing(&self) -> &render::draw::ScaffoldTiming {
        &self.timing
    }

    /// `constructsize` of the piece's `<BUILD>`; 0 where it has none.
    pub fn construct_size(&self, piece: i32) -> i32 {
        self.construct.get(&piece).copied().unwrap_or(0)
    }

    /// **The foundation `WallOut::draw_underlays@006410a0` draws under a
    /// building of `type_index` showing `piece`**, or `None` for the farm
    /// (`!= 0x1a1` at `0064154c`) and a type [`BuildLook::with_types`] did
    /// not see. The slot is `GraphicPieces::get_construct_gpiece
    /// @00907840`'s: 9 for the water domain, 10 for the Senate's lineage,
    /// else the piece's `constructsize`, or the footprint's `y` where that
    /// is 0, less one. The point is moved back the piece's `halfoffset`
    /// half tiles, or, where its `constructsize` is not 0, the footprint's
    /// `x` less it (`draw_underlays`' reads of the piece's `BuildRData
    /// +0x28` and `+0x2c`).
    pub fn foundation(&self, type_index: i32, piece: i32) -> Option<render::draw::Foundation> {
        let k = self.kinds.get(&type_index)?;
        if k.farm {
            return None;
        }
        let construct = self.construct_size(piece);
        let slot = if k.water {
            9
        } else if k.senate {
            10
        } else if construct != 0 {
            construct - 1
        } else {
            k.y_size - 1
        };
        let back = if construct != 0 {
            k.x_size - construct
        } else {
            self.half_offset(type_index)
        };
        Some(render::draw::Foundation {
            slot: u8::try_from(slot).ok()?,
            back,
        })
    }

    /// **The `<SCAFFOLD_TIMING>` table `GraphicPieces::draw_scaffold
    /// @00907420` reads for `type_index`**: the footprint's `y` less one,
    /// or 9 for the Senate's lineage; `None` for the farm, the water
    /// domain and a type [`BuildLook::with_types`] did not see.
    pub fn scaffold_table(&self, type_index: i32) -> Option<u8> {
        let k = self.kinds.get(&type_index)?;
        if k.farm || k.water {
            return None;
        }
        u8::try_from(if k.senate { 9 } else { k.y_size - 1 }).ok()
    }

    /// `halfoffset` of the type's `<BUILD>` entries; 0 where it has none.
    pub fn half_offset(&self, type_index: i32) -> i32 {
        self.half.get(&type_index).copied().unwrap_or(0)
    }

    /// **`GraphicPieces::get_build_gpiece@0090bce0`**, as `Wall::update_gpiece
    /// @00643070` calls it for a building (`param_4` its `max_age` less its
    /// leader's age, so the walk starts at the building's own age): from
    /// the age, clamped to 0..=7, down to 0 in the nation's style; then,
    /// where the backup style differs, down again in it; then down in the
    /// default style; the first piece that loads. Nothing loading is the
    /// type's age-0 default piece.
    pub fn piece(&self, type_index: i32, style: i32, backup: i32, age: i32) -> i32 {
        let ty = type_index - 0x19e;
        let at = |style: i32, age: i32| {
            FIRST_BUILD_PIECE + NUM_BUILD_PIECES * style + BUILD_PIECES_PER_AGE * age + ty
        };
        let top = age.clamp(0, 7);
        let mut styles = vec![style];
        if backup != style {
            styles.push(backup);
        }
        styles.push(0);
        styles
            .into_iter()
            .flat_map(|s| (0..=top).rev().map(move |a| at(s, a)))
            .find(|p| self.loads.contains(p))
            .unwrap_or(FIRST_BUILD_PIECE + ty)
    }
}

/// **`GraphicPieces::init_scaffold_timing@008ed6c0`'s ten tables**
/// (item 4061): `effects_graphics.xml`'s `<SCAFFOLD_TIMING>` elements in
/// file order, each `<TIMING name start_percent end_percent x y z angle>`
/// a slot, `x`, `y` and `z` times three (`0x40400000`). A slot is kept
/// only where its `name` is one of `ammo_names` — the comparison is
/// `String::operator==@00a1f140`'s, without case, so `Crane` finds
/// `crane` — and its element is the name's first `<SCAFFOLD>`, in file
/// order: the loader's `first_ammo_piece + index`, less the scaffold's
/// first piece. That holds while no `<AMMO>` or `<ENTRENCH>` shares a
/// scaffold's name, which this refuses; it also refuses an offset or an
/// angle that is not whole.
pub fn scaffold_timing(install: &Install) -> Result<render::draw::ScaffoldTiming, String> {
    let path = install.data("effects_graphics.xml");
    let text = crate::read(&path).map_err(|e| e.to_string())?;
    let doc = crate::parse(&path, &text).map_err(|e| e.to_string())?;
    let name = |n: &roxmltree::Node<'_, '_>| {
        n.attribute("name")
            .map(|v| v.trim().to_ascii_lowercase())
            .unwrap_or_default()
    };
    let mut scaffolds: Vec<String> = Vec::new();
    for n in doc.descendants().filter(|n| n.has_tag_name("SCAFFOLD")) {
        scaffolds.push(name(&n));
    }
    for n in doc
        .descendants()
        .filter(|n| n.has_tag_name("AMMO") || n.has_tag_name("ENTRENCH"))
    {
        if scaffolds.contains(&name(&n)) {
            return Err(format!("{} shares a scaffold's name", name(&n)));
        }
    }
    let whole = |n: &roxmltree::Node<'_, '_>, key: &str, by: i32| -> Result<i32, String> {
        let v = n.attribute(key).map(str::trim).unwrap_or("0");
        v.parse::<i32>()
            .map(|v| v * by)
            .map_err(|_| format!("<TIMING {key}=\"{v}\"> is not whole"))
    };
    let share = |n: &roxmltree::Node<'_, '_>, key: &str| -> f32 {
        n.attribute(key)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or(0.0) as f32
    };
    let mut tables = Vec::new();
    for t in doc
        .descendants()
        .filter(|n| n.has_tag_name("SCAFFOLD_TIMING"))
    {
        let mut slots = Vec::new();
        for s in t.children().filter(|n| n.has_tag_name("TIMING")) {
            let Some(element) = scaffolds.iter().position(|x| *x == name(&s)) else {
                continue;
            };
            slots.push(render::draw::ScaffoldSlot {
                element: u8::try_from(element).map_err(|e| e.to_string())?,
                start: share(&s, "start_percent"),
                end: share(&s, "end_percent"),
                dx: whole(&s, "x", 3)?,
                dy: whole(&s, "y", 3)?,
                dz: whole(&s, "z", 3)?,
                angle: whole(&s, "angle", 1)?,
            });
        }
        tables.push(slots);
    }
    Ok(render::draw::ScaffoldTiming { tables })
}

/// One `<CONSTRUCTION>` of `building_graphics.xml` (item 4163): the
/// construction piece `GraphicPieces::get_construct_gpiece@00907840`'s slot
/// names, in file order. `get_gpiece_type@009096e0` puts the construct
/// range at type `0xc`, which `GraphicPieces::draw@0090b2d0` sends to
/// `draw_build` — a sprite, as a building's (`texture=`, anchored back from
/// the projected point by `spritexoff`, `spriteyoff`; [`BuildArt`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructionArt {
    pub texture: String,
    pub sprite_offset: (i32, i32),
    /// `model=`: only the dock's names one.
    pub model: Option<String>,
}

/// The `<CONSTRUCTION>` entries whose files decode, of all (item 4163:
/// new, 11 of 11 — the ten sprites and the dock's sprite and model).
pub const CONSTRUCTION_DECODING: (usize, usize) = (11, 11);

/// The `<SCAFFOLD>` entries whose model, texture and animation decode, of
/// all (item 4163: new, 12 of 12 — four pieces an age bracket, the crane's
/// `CraneWork` animation among them).
pub const SCAFFOLD_DECODING: (usize, usize) = (12, 12);

/// **The eleven `<CONSTRUCTION>` entries**, in file order (item 4163;
/// [`render::draw::Foundation::slot`] indexes them).
pub fn construction_art(install: &Install) -> Vec<ConstructionArt> {
    let path = install.data("building_graphics.xml");
    let Ok(text) = crate::read(&path) else {
        return Vec::new();
    };
    let Ok(doc) = crate::parse(&path, &text) else {
        return Vec::new();
    };
    doc.descendants()
        .filter(|n| n.has_tag_name("CONSTRUCTION"))
        .map(|n| {
            let named = |a: &str| {
                n.attribute(a)
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string)
            };
            let num = |a: &str| named(a).and_then(|v| v.parse().ok()).unwrap_or(-1);
            ConstructionArt {
                texture: named("texture").unwrap_or_default(),
                sprite_offset: (num("spritexoff"), num("spriteyoff")),
                model: named("model"),
            }
        })
        .collect()
}

/// One `<SCAFFOLD>` of `effects_graphics.xml` (item 4163): the ammo-range
/// piece `GraphicPieces::draw_scaffold@00907420` draws by element
/// ([`render::draw::ScaffoldPiece::element`]), a model under a texture,
/// and an `anim=` through `anim_graphics.xml` — `(name, file)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScaffoldArt {
    pub name: String,
    pub texture: String,
    pub model: String,
    pub anim: Option<(String, Option<String>)>,
}

/// **The `<SCAFFOLD>` entries**, in file order — four an age bracket
/// (item 4163; [`scaffold_timing`] numbers its elements on the same walk).
pub fn scaffold_art(install: &Install) -> Vec<ScaffoldArt> {
    let (epath, apath) = (
        install.data("effects_graphics.xml"),
        install.data("anim_graphics.xml"),
    );
    let (Ok(etext), Ok(atext)) = (crate::read(&epath), crate::read(&apath)) else {
        return Vec::new();
    };
    let (Ok(edoc), Ok(adoc)) = (crate::parse(&epath, &etext), crate::parse(&apath, &atext)) else {
        return Vec::new();
    };
    let files = anim_files(&adoc);
    edoc.descendants()
        .filter(|n| n.has_tag_name("SCAFFOLD"))
        .map(|n| {
            let named = |a: &str| {
                n.attribute(a)
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string)
            };
            ScaffoldArt {
                name: named("name").unwrap_or_default(),
                texture: named("texture").unwrap_or_default(),
                model: named("model").unwrap_or_default(),
                anim: named("anim").map(|a| {
                    let f = files.get(&anim_key(&a)).map(|(f, _)| f.clone());
                    (a, f)
                }),
            }
        })
        .collect()
}

/// **A building piece the loader reads as a `<UNIT>`** (item 4169,
/// `docs/RENDERER.md` §3.13): `GraphicPieces::verify_load@00906550:162`–`173`
/// sends a build piece of type `0x20b`, `0x20c` or `0x20d` (the Air Defense
/// Gun, Radar Air Defense and SAM Installation) to `init_unit_data` rather
/// than `init_build_data` when it is one of the first `0x81` build pieces —
/// the default style's age 0. Its `<BUILD>` names no file; the `<UNIT>` of
/// the same name ([`build_piece_name`], `GRAPH-DEFAULT-AGE0`) is its art,
/// as [`wall_packets`] reads its slots.
pub fn loads_as_unit(piece: i32, type_index: i32) -> bool {
    (0x20b..=0x20d).contains(&type_index)
        && (0..NUM_BUILD_PIECES).contains(&(piece - FIRST_BUILD_PIECE))
}

/// **A building piece's condition 1** (item 4028, `docs/RENDERER.md`
/// §5.4): every model and animation its [`BuildArt`] names — the
/// `zbuffer` `.bh3`, the building's own `anim`, each addon's `.bh3` and
/// `.bha` — decodes under `render`'s readers. `Ok(files decoded)`, or the
/// failing files; a piece with no `<BUILD>` fails as `Unnamed` its name.
/// The farm's pieces ([`FARM_TYPE`]) are held, beside their `<BUILD>`, to
/// the files `FarmsOut::draw@008d75f0` draws them from on `tileset`
/// ([`farm_decoding`], item 4037).
///
/// **An animation name `anim_graphics.xml` does not carry names no file**
/// (item 4169): `GraphicPieces::decipher_animation@008face0` answers `-1`,
/// and neither building loader reports it. `init_build_data@008f2f20:295`
/// reads the `<BUILD>`'s own `anim` only for `build_anim_time`, skipping a
/// `-1` and leaving the time 0; `init_build_events@008ea3c0:252` stores the
/// `-1` in the event and carries on. So the piece loads, and nothing is
/// opened for that name. A name that is carried and whose file the install
/// lacks still fails as `Missing`.
///
/// A piece that [`loads_as_unit`] is held to its `<UNIT>`'s model and
/// every slot's file, as [`piece_decoding`] holds a figure's.
pub fn building_decoding(
    install: &Install,
    pieces: &std::collections::BTreeSet<i32>,
    tileset: &str,
) -> BTreeMap<i32, Result<usize, Vec<ArtFailure>>> {
    let read = |rel: &str| -> Result<Vec<u8>, ArtFailure> {
        let path = resolve(install.root(), rel).ok_or_else(|| ArtFailure::Missing(rel.into()))?;
        std::fs::read(path).map_err(|_| ArtFailure::Missing(rel.into()))
    };
    let model = |rel: &str| -> Result<(), ArtFailure> {
        render::bh3::decode(&read(rel)?)
            .map(|_| ())
            .map_err(|e| ArtFailure::Undecodable(rel.into(), e.to_string()))
    };
    let anim = |rel: &str| -> Result<(), ArtFailure> {
        render::bha::decode(&read(rel)?)
            .map(|_| ())
            .map_err(|e| ArtFailure::Undecodable(rel.into(), e.to_string()))
    };
    let mut out = BTreeMap::new();
    let mut units: Option<Vec<UnitArt>> = None;
    for (piece, art) in build_art(install, pieces) {
        let art = match art {
            Ok(art) => art,
            Err(name) => {
                out.insert(piece, Err(vec![ArtFailure::Unnamed(name)]));
                continue;
            }
        };
        let (mut n, mut fails) = (0, Vec::new());
        let mut check = |r: Result<(), ArtFailure>| match r {
            Ok(()) => n += 1,
            Err(f) => fails.push(f),
        };
        if let Some(z) = &art.zbuffer {
            check(model(z));
        }
        if let Some((_, Some(f))) = &art.anim {
            check(anim(f));
        }
        for (subject, m, _, f) in &art.addons {
            check(
                m.as_deref()
                    .map_or(Err(ArtFailure::Unnamed(subject.clone())), model),
            );
            if let Some(f) = f {
                check(anim(f));
            }
        }
        if loads_as_unit(piece, art.type_index) {
            let units = units.get_or_insert_with(|| unit_art(install, &[]));
            match units
                .iter()
                .find(|u| u.name.eq_ignore_ascii_case(&art.name))
            {
                None => check(Err(ArtFailure::Unnamed(art.name.clone()))),
                Some(u) => {
                    check(model(&u.model));
                    for (_, a, f) in &u.anims {
                        check(
                            f.as_deref()
                                .map_or(Err(ArtFailure::Unnamed(a.clone())), anim),
                        );
                    }
                }
            }
        }
        if art.type_index == FARM_TYPE {
            match farm_decoding(install, tileset) {
                Ok(k) => n += k,
                Err(f) => fails.extend(f),
            }
        }
        out.insert(piece, if fails.is_empty() { Ok(n) } else { Err(fails) });
    }
    out
}

/// **The farm's type**, `TypeIndex 0x1a1` (417): the type
/// `Build::init@00629740` hands to `Farms::add@008d8a40`, so the type
/// `FarmsOut::draw@008d75f0` draws (item 4037).
pub const FARM_TYPE: i32 = 0x1a1;

/// **The farm's art, as the loaders hold it** (item 4037, `docs/FORMATS.md`,
/// "The gaia pieces and the building pieces"). A farm is not a mesh at a point: its `<BUILD>`
/// (`FARM-DEFAULT-AGE0`) names no model, and `FarmsOut::draw@008d75f0`
/// builds a wheat field over the farm's footprint each frame — stalks of
/// the `<FARM>` element's colour, between its two heights, textured with
/// the tileset's wheat image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FarmArt {
    /// `stalk_red`, `stalk_green`, `stalk_blue`, `stalk_alpha` — strings
    /// 2674–2677, which `Farms::init@008d9310` reads with
    /// `get_attrib_num` into four bytes at `farms + 0x40`.
    pub stalk: [u8; 4],
    /// `min_height`, `max_height` (strings 2678, 2679), as written: the
    /// loader reads them with `get_attrib_float` and keeps each × 3.0
    /// (`farms + 0x50`, `+ 0x4c`).
    pub heights: [i32; 2],
    /// The `<FARM>`'s own `texture=` (string 136): `Farms::init` reads it
    /// into the scratch string `msg` and nothing loads it.
    pub unread_texture: String,
    /// The tileset's `FARMTEX/TEXTURE` `file=` (strings 6766, 743), which
    /// `TileSet::init@0087b370` gives `TileSetData::wheat_texture` and
    /// `FarmsOut::draw` binds to stage 0.
    pub wheat: String,
}

/// [`FarmArt`] for `tileset` (a `<TILESET name>` in `Data/tilesets.xml`,
/// compared without case): `Root/Buildings/Farm` in
/// `building_graphics.xml`, walked as `Farms::init@008d9310` walks it
/// (element names without case), and the tileset's `FARMTEX`. A missing
/// element or attribute, or one that does not parse, is `Unnamed` its
/// name — `get_attrib_num`'s `-1` default is not a value the file wrote.
pub fn farm_art(install: &Install, tileset: &str) -> Result<FarmArt, ArtFailure> {
    let unnamed = |what: &str| ArtFailure::Unnamed(what.to_string());
    fn child<'a, 'i>(n: roxmltree::Node<'a, 'i>, tag: &str) -> Option<roxmltree::Node<'a, 'i>> {
        n.children()
            .find(|c| c.is_element() && c.tag_name().name().eq_ignore_ascii_case(tag))
    }
    let bpath = install.data("building_graphics.xml");
    let btext = crate::read(&bpath).map_err(|_| unnamed("building_graphics.xml"))?;
    let bdoc = crate::parse(&bpath, &btext).map_err(|_| unnamed("building_graphics.xml"))?;
    let root = bdoc.root_element();
    let farm = child(root, "Buildings")
        .and_then(|b| child(b, "Farm"))
        .filter(|_| root.tag_name().name().eq_ignore_ascii_case("Root"))
        .ok_or_else(|| unnamed("Root/Buildings/Farm"))?;
    let num = |a: &str| -> Result<i32, ArtFailure> {
        farm.attribute(a)
            .and_then(|v| v.trim().parse().ok())
            .ok_or_else(|| unnamed(a))
    };
    let byte =
        |a: &str| -> Result<u8, ArtFailure> { u8::try_from(num(a)?).map_err(|_| unnamed(a)) };
    let stalk = [
        byte("stalk_red")?,
        byte("stalk_green")?,
        byte("stalk_blue")?,
        byte("stalk_alpha")?,
    ];
    let heights = [num("min_height")?, num("max_height")?];
    let tpath = install.data("tilesets.xml");
    let ttext = crate::read(&tpath).map_err(|_| unnamed("tilesets.xml"))?;
    let tdoc = crate::parse(&tpath, &ttext).map_err(|_| unnamed("tilesets.xml"))?;
    let wheat = tdoc
        .descendants()
        .find(|n| {
            n.has_tag_name("TILESET")
                && n.attribute("name")
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case(tileset))
        })
        .and_then(|t| child(t, "FARMTEX"))
        .and_then(|f| child(f, "TEXTURE"))
        .and_then(|t| t.attribute("file"))
        .map(|f| f.trim().to_string())
        .filter(|f| !f.is_empty())
        .ok_or_else(|| unnamed(&format!("{tileset}/FARMTEX/TEXTURE")))?;
    Ok(FarmArt {
        stalk,
        heights,
        unread_texture: farm
            .attribute("texture")
            .unwrap_or_default()
            .trim()
            .to_string(),
        wheat,
    })
}

/// **The map's trees, as the loaders hold them** (item 4080,
/// `docs/RENDERER.md` §3.7): the tileset's `<TREES treescale=…>`, which
/// `TileSet::init@0087b370` multiplies by 3 into `TileSetData::tree_scale`;
/// its `<BASEDIR path=…>`; and the models `GraphicPieces::init_tree_data
/// @008eeb30` reads from `graphics.txt`'s `#<TILESET>_TREE_MODELS0` section,
/// one file a line under the base directory, `;` a line left out.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeArt {
    pub treescale: f32,
    /// Each model as an install path: the base directory joined to the
    /// section's line.
    pub models: Vec<String>,
    /// **The seven textures a tree's sections sample** (item 4082,
    /// `docs/RENDERER.md` §3.7), as install paths: the first seven lines of
    /// `#<TILESET>_TREE_TEXTURES0`, each `name, path`, in the order
    /// `init_tree_data` hands them to `RTreeData` — the trunk, then the
    /// bottom, middle and top leaves. The eighth, the shadow, is left out.
    pub textures: Vec<String>,
    /// The section's eighth line, `Shadow Texture` (item 4182): the decal
    /// `GraphicPieces::draw_tree_shadow@00905910` lays under each tree,
    /// the card's draw 119 on run1566b's pass after 3279. `None` where the
    /// section has no eighth line.
    pub shadow: Option<String>,
}

/// [`TreeArt`] for `tileset`. **The section's name is a reading**: the
/// upper-cased tileset name and `_TREE_MODELS0`, its count checked against
/// the packet's 24 on `early_autumn` (§3.7), not read off the string table.
pub fn tree_art(install: &Install, tileset: &str) -> Result<TreeArt, ArtFailure> {
    let unnamed = |what: &str| ArtFailure::Unnamed(what.to_string());
    let tpath = install.data("tilesets.xml");
    let ttext = crate::read(&tpath).map_err(|_| unnamed("tilesets.xml"))?;
    let tdoc = crate::parse(&tpath, &ttext).map_err(|_| unnamed("tilesets.xml"))?;
    let set = tdoc
        .descendants()
        .find(|n| {
            n.has_tag_name("TILESET")
                && n.attribute("name")
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case(tileset))
        })
        .ok_or_else(|| unnamed(tileset))?;
    let child = |tag: &str| {
        set.children()
            .find(|c| c.is_element() && c.tag_name().name().eq_ignore_ascii_case(tag))
    };
    let treescale = child("TREES")
        .and_then(|t| t.attribute("treescale"))
        .and_then(|v| v.trim().parse::<f32>().ok())
        .ok_or_else(|| unnamed(&format!("{tileset}/TREES/treescale")))?;
    let base = child("BASEDIR")
        .and_then(|b| b.attribute("path"))
        .map(|p| p.trim().replace('\\', "/"))
        .ok_or_else(|| unnamed(&format!("{tileset}/BASEDIR")))?;
    let gpath = install.root().join("graphics.txt");
    let gtext = std::fs::read(&gpath).map_err(|_| unnamed("graphics.txt"))?;
    let gtext = String::from_utf8_lossy(&gtext);
    let lines = |name: &str| -> Vec<String> {
        gtext
            .lines()
            .skip_while(|l| !l.trim().eq_ignore_ascii_case(name))
            .skip(1)
            .take_while(|l| !l.trim_start().starts_with('#'))
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with(';'))
            .map(str::to_string)
            .collect()
    };
    let path = |l: &str| format!("{}/{}", base.trim_end_matches('/'), l.replace('\\', "/"));
    let section = format!("#{}_TREE_MODELS0", tileset.to_ascii_uppercase());
    let models: Vec<String> = lines(&section).iter().map(|l| path(l)).collect();
    if models.is_empty() {
        return Err(unnamed(&section));
    }
    let tsection = format!("#{}_TREE_TEXTURES0", tileset.to_ascii_uppercase());
    let mut textures: Vec<String> = lines(&tsection)
        .iter()
        .take(8)
        .filter_map(|l| l.split_once(',').map(|(_, f)| path(f.trim())))
        .collect();
    let shadow = (textures.len() == 8).then(|| textures.remove(7));
    if textures.len() != 7 {
        return Err(unnamed(&tsection));
    }
    Ok(TreeArt {
        treescale,
        models,
        textures,
        shadow,
    })
}

/// **The ground's base layer, as the loaders hold it** (item 4127,
/// `docs/RENDERER.md` §3.6, "The ground's base layer"): the tileset's
/// `<BASEDIR path=…>` and its `<BASELAND>`'s `<BASE>`s in file order —
/// the order `TileSet::convert_land_tex_to_linear@0087ace0` counts them in,
/// so a `BASELAND` cell's `land_sub` indexes this list — each its
/// `<TEXTURE name=…>` as an install path under the base directory, and its
/// `supertile=` (a supertile is cut 3 × 3 by
/// `TextureEffect::load_as_supertile@00542640`).
#[derive(Clone, Debug, PartialEq)]
pub struct GroundArt {
    pub baseland: Vec<GroundBase>,
    /// `<COASTAL>`'s and `<OCEAN>`'s `<BASE>`s (item 4164): the land types
    /// `eTILE_COASTAL` (1) and `eTILE_OCEAN` (2) that
    /// `convert_land_tex_to_linear` counts after `BASELAND`'s. A `SANDY` or
    /// an `OCEAN` cell takes its type's first texture, with no sub-tile.
    pub coastal: Vec<GroundBase>,
    pub ocean: Vec<GroundBase>,
}

/// One `<BASE>` under a tileset's `<BASELAND>`.
#[derive(Clone, Debug, PartialEq)]
pub struct GroundBase {
    pub name: String,
    /// The `<TEXTURE name=…>` joined to the tileset's base directory.
    pub texture: String,
    pub supertile: bool,
}

/// [`GroundArt`] for `tileset` (a `<TILESET name>`, compared without case).
pub fn ground_art(install: &Install, tileset: &str) -> Result<GroundArt, ArtFailure> {
    let unnamed = |what: &str| ArtFailure::Unnamed(what.to_string());
    let tpath = install.data("tilesets.xml");
    let ttext = crate::read(&tpath).map_err(|_| unnamed("tilesets.xml"))?;
    let tdoc = crate::parse(&tpath, &ttext).map_err(|_| unnamed("tilesets.xml"))?;
    let set = tdoc
        .descendants()
        .find(|n| {
            n.has_tag_name("TILESET")
                && n.attribute("name")
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case(tileset))
        })
        .ok_or_else(|| unnamed(tileset))?;
    fn child<'a, 'i>(n: roxmltree::Node<'a, 'i>, tag: &str) -> Option<roxmltree::Node<'a, 'i>> {
        n.children()
            .find(|c| c.is_element() && c.tag_name().name().eq_ignore_ascii_case(tag))
    }
    let base = child(set, "BASEDIR")
        .and_then(|b| b.attribute("path"))
        .map(|p| p.trim().replace('\\', "/"))
        .ok_or_else(|| unnamed(&format!("{tileset}/BASEDIR")))?;
    let bases = |tag: &str| -> Result<Vec<GroundBase>, ArtFailure> {
        let Some(land) = child(set, tag) else {
            return Ok(Vec::new());
        };
        land.children()
            .filter(|c| c.is_element() && c.tag_name().name().eq_ignore_ascii_case("BASE"))
            .map(|b| {
                let name = b.attribute("name").unwrap_or_default().trim().to_string();
                let texture = child(b, "TEXTURE")
                    .and_then(|t| t.attribute("name"))
                    .map(|f| {
                        format!(
                            "{}/{}",
                            base.trim_end_matches('/'),
                            f.trim().replace('\\', "/")
                        )
                    })
                    .ok_or_else(|| unnamed(&format!("{tileset}/{tag}/{name}/TEXTURE")))?;
                let supertile = b.attribute("supertile").is_some_and(|v| v.trim() == "1");
                Ok(GroundBase {
                    name,
                    texture,
                    supertile,
                })
            })
            .collect()
    };
    child(set, "BASELAND").ok_or_else(|| unnamed(&format!("{tileset}/BASELAND")))?;
    let baseland = bases("BASELAND")?;
    if baseland.is_empty() {
        return Err(unnamed(&format!("{tileset}/BASELAND/BASE")));
    }
    Ok(GroundArt {
        baseland,
        coastal: bases("COASTAL")?,
        ocean: bases("OCEAN")?,
    })
}

/// **The tileset's mountain textures** (item 4141; `docs/RENDERER.md`
/// §3.10): `<MOUNTAINS>`'s `BASETEX` and `ALPHATEX` files under
/// `tileset`, as written. `TileSet::init` loads them as
/// `TileSetData::mountain_base` and `mountain_alpha`, the two stage 0
/// textures `TerrainOut::render_mountains` binds.
pub fn mountain_textures(install: &Install, tileset: &str) -> Result<(String, String), ArtFailure> {
    let unnamed = |what: &str| ArtFailure::Unnamed(what.to_string());
    let tpath = install.data("tilesets.xml");
    let ttext = crate::read(&tpath).map_err(|_| unnamed("tilesets.xml"))?;
    let tdoc = crate::parse(&tpath, &ttext).map_err(|_| unnamed("tilesets.xml"))?;
    let set = tdoc
        .descendants()
        .find(|n| {
            n.has_tag_name("TILESET")
                && n.attribute("name")
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case(tileset))
        })
        .ok_or_else(|| unnamed(tileset))?;
    let m = set
        .children()
        .find(|n| n.has_tag_name("MOUNTAINS"))
        .ok_or_else(|| unnamed(&format!("{tileset}/MOUNTAINS")))?;
    let file = |tag: &str| {
        m.children()
            .find(|n| n.has_tag_name(tag))
            .and_then(|n| n.attribute("file"))
            .map(|f| f.trim().to_string())
            .ok_or_else(|| unnamed(&format!("{tileset}/MOUNTAINS/{tag}")))
    };
    Ok((file("BASETEX")?, file("ALPHATEX")?))
}

/// **The tileset's coral texture** (item 4200; `docs/RENDERER.md` §3.18):
/// `<CORALTEX>`'s `TEXTURE` `file=` under `tileset`, as written.
/// `TileSet::init@0087b370` loads it as `TileSetData::coral_tex`, the
/// texture `TerrainOut::render` binds over every patch drawn in
/// `oceanindex`'s texture and `TerrainOut::render_coral` over the sand
/// layer. `None` where the tileset names none.
pub fn coral_texture(install: &Install, tileset: &str) -> Result<Option<String>, ArtFailure> {
    let unnamed = |what: &str| ArtFailure::Unnamed(what.to_string());
    let tpath = install.data("tilesets.xml");
    let ttext = crate::read(&tpath).map_err(|_| unnamed("tilesets.xml"))?;
    let tdoc = crate::parse(&tpath, &ttext).map_err(|_| unnamed("tilesets.xml"))?;
    let set = tdoc
        .descendants()
        .find(|n| {
            n.has_tag_name("TILESET")
                && n.attribute("name")
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case(tileset))
        })
        .ok_or_else(|| unnamed(tileset))?;
    Ok(set
        .children()
        .find(|n| n.has_tag_name("CORALTEX"))
        .and_then(|c| c.children().find(|n| n.has_tag_name("TEXTURE")))
        .and_then(|t| t.attribute("file"))
        .map(|f| f.trim().to_string()))
}

/// **The tileset's tint** (item 4212; `docs/RENDERER.md` §3.21): the
/// `<LIGHTING>` element's `tint_red`, `tint_green` and `tint_blue` under
/// `tileset`, as `TileSet::init@0087b370` reads them into
/// `TileSetData::tint_color`. `ImageIO::set_tint@00511a00` hands them to
/// every tileset texture's loader, and `ImageIO::tint@00511aa0` adds them,
/// saturating, to each decoded texel's bytes. A missing attribute is 0, as
/// `get_attrib_num`'s default of −1 truncated to a byte would not be: the
/// install writes all three on every tileset.
pub fn tileset_tint(install: &Install, tileset: &str) -> Result<[u8; 3], ArtFailure> {
    let unnamed = |what: &str| ArtFailure::Unnamed(what.to_string());
    let tpath = install.data("tilesets.xml");
    let ttext = crate::read(&tpath).map_err(|_| unnamed("tilesets.xml"))?;
    let tdoc = crate::parse(&tpath, &ttext).map_err(|_| unnamed("tilesets.xml"))?;
    let set = tdoc
        .descendants()
        .find(|n| {
            n.has_tag_name("TILESET")
                && n.attribute("name")
                    .is_some_and(|v| v.trim().eq_ignore_ascii_case(tileset))
        })
        .ok_or_else(|| unnamed(tileset))?;
    let light = set
        .children()
        .find(|n| n.has_tag_name("LIGHTING"))
        .ok_or_else(|| unnamed(&format!("{tileset}/LIGHTING")))?;
    let mut out = [0u8; 3];
    for (o, a) in out.iter_mut().zip(["tint_red", "tint_green", "tint_blue"]) {
        let v = light
            .attribute(a)
            .ok_or_else(|| unnamed(&format!("{tileset}/LIGHTING/{a}")))?;
        *o = v
            .trim()
            .parse()
            .map_err(|_| unnamed(&format!("{tileset}/LIGHTING/{a}")))?;
    }
    Ok(out)
}

/// **The footprint weathering's textures** (item 4190; `docs/RENDERER.md`
/// §3.17): `Data/tilesets.xml`'s `<COMMON><WEATHERING>` `WEATHERELEM`s, one
/// per age bucket as `age=` numbers them, each `(file1, file2)` as written.
/// `Terrain::add_weathering@00850120` binds the bucket's `file2`
/// (`weather_textures_fertile_0[bucket]`) as stage 0; the `ALPHAELEM`s are
/// stage 1, which the card's draw 90 leaves disabled, so they are not read.
pub fn weathering_art(install: &Install) -> Result<Vec<(i32, String, String)>, ArtFailure> {
    let unnamed = |what: &str| ArtFailure::Unnamed(what.to_string());
    let tpath = install.data("tilesets.xml");
    let ttext = crate::read(&tpath).map_err(|_| unnamed("tilesets.xml"))?;
    let tdoc = crate::parse(&tpath, &ttext).map_err(|_| unnamed("tilesets.xml"))?;
    let w = tdoc
        .descendants()
        .find(|n| n.has_tag_name("COMMON"))
        .and_then(|c| c.children().find(|n| n.has_tag_name("WEATHERING")))
        .ok_or_else(|| unnamed("COMMON/WEATHERING"))?;
    let mut out = Vec::new();
    for e in w.children().filter(|n| n.has_tag_name("WEATHERELEM")) {
        let field = |a: &str| {
            e.attribute(a)
                .map(|v| v.trim().to_string())
                .ok_or_else(|| unnamed(&format!("WEATHERELEM/{a}")))
        };
        let age = field("age")?
            .parse()
            .map_err(|_| unnamed("WEATHERELEM/age"))?;
        out.push((age, field("file1")?, field("file2")?));
    }
    Ok(out)
}

/// **One road piece** (item 4199, `docs/RENDERER.md` §3.19): a
/// `<PAGEELEMENT>` of `Data/roads.xml`, as `RoadsPieces::parse_element
/// @00895fc0` reads it into a `RoadElement`. Sizes and offsets are in half
/// tiles, as the file's comments say; the four flag bytes are its bit
/// strings, the first character bit 7 (NW, then clockwise).
#[derive(Clone, Debug, PartialEq)]
pub struct RoadPiece {
    pub name: String,
    /// The `<ROADPAGE>`'s index, `RoadElement::pagenum`.
    pub page: u8,
    pub width: u8,
    pub height: u8,
    pub offset: (u8, u8),
    /// `<CENTEROFFSET>`'s eight `<OFFSET>`s, one per rotation.
    pub center: [(f32, f32); 8],
    pub road: u8,
    pub yellow: u8,
    pub white: u8,
    pub intersect: u8,
}

/// **The road pieces and their pages** (item 4199, §3.19): every
/// `<ROADPAGE>`'s `<PAGEELEMENT>`s in file order — `RoadsPieces::
/// modern_elements`, whose index is a record's `element_num` — and each
/// page's `<TEXTURE>`s as `(age, name)`.
#[derive(Clone, Debug, PartialEq)]
pub struct RoadArt {
    pub pieces: Vec<RoadPiece>,
    pub pages: Vec<Vec<(i32, String)>>,
}

impl RoadArt {
    /// The index of the piece a name names, as `parse_element` remembers
    /// `C4` (`RoadsPieces::element_C4`) and `C31` (`?C31@@3HA`, the piece
    /// `RoadsPieces::find_map@00897840` takes when nothing matches).
    pub fn named(&self, name: &str) -> Option<usize> {
        self.pieces
            .iter()
            .position(|p| p.name.eq_ignore_ascii_case(name))
    }
}

/// `Data/roads.xml`, read whole ([`RoadArt`]).
pub fn road_art(install: &Install) -> Result<RoadArt, ArtFailure> {
    let unnamed = |what: &str| ArtFailure::Unnamed(what.to_string());
    let path = install.data("roads.xml");
    let text = crate::read(&path).map_err(|_| unnamed("roads.xml"))?;
    let doc = crate::parse(&path, &text).map_err(|_| unnamed("roads.xml"))?;
    let mut art = RoadArt {
        pieces: Vec::new(),
        pages: Vec::new(),
    };
    let pages = doc.descendants().filter(|n| n.has_tag_name("ROADPAGE"));
    for (page, pg) in pages.enumerate() {
        let mut textures = Vec::new();
        for t in pg.children().filter(|n| n.has_tag_name("TEXTURE")) {
            let age = t
                .attribute("age")
                .and_then(|a| a.trim().parse().ok())
                .ok_or_else(|| unnamed("TEXTURE/age"))?;
            let name = t.attribute("name").ok_or_else(|| unnamed("TEXTURE/name"))?;
            textures.push((age, name.trim().to_string()));
        }
        art.pages.push(textures);
        for e in pg.children().filter(|n| n.has_tag_name("PAGEELEMENT")) {
            let child = |tag: &str| e.children().find(|n| n.has_tag_name(tag));
            let number = |tag: &str| -> Result<u8, ArtFailure> {
                child(tag)
                    .and_then(|n| n.text())
                    .and_then(|t| t.trim().parse().ok())
                    .ok_or_else(|| unnamed(&format!("PAGEELEMENT/{tag}")))
            };
            let bits = |tag: &str| -> Result<u8, ArtFailure> {
                child(tag)
                    .and_then(|n| n.attribute("value"))
                    .and_then(|v| u8::from_str_radix(v.trim(), 2).ok())
                    .ok_or_else(|| unnamed(&format!("PAGEELEMENT/{tag}")))
            };
            let mut center = [(0.0, 0.0); 8];
            let offs: Vec<_> = child("CENTEROFFSET")
                .map(|c| c.children().filter(|n| n.has_tag_name("OFFSET")).collect())
                .unwrap_or_default();
            for (k, o) in offs.iter().take(8).enumerate() {
                let f = |a: &str| o.attribute(a).and_then(|v| v.trim().parse::<f32>().ok());
                center[k] = (f("x").unwrap_or(0.0), f("y").unwrap_or(0.0));
            }
            art.pieces.push(RoadPiece {
                name: e.attribute("name").unwrap_or_default().trim().to_string(),
                page: page as u8,
                width: number("WIDTH")?,
                height: number("HEIGHT")?,
                offset: (number("OFFSETX")?, number("OFFSETY")?),
                center,
                road: bits("ROADFLAGS")?,
                yellow: bits("YELLOWFLAGS")?,
                white: bits("WHITEFLAGS")?,
                intersect: bits("INTERSECTFLAGS")?,
            });
        }
    }
    Ok(art)
}

/// **The farm's condition 1** (item 4037, `docs/RENDERER.md` §5.4): the
/// `<FARM>` element reads whole ([`farm_art`]) and the tileset's wheat
/// image decodes under `render::tga`. `Ok(1)`, the one file decoded.
pub fn farm_decoding(install: &Install, tileset: &str) -> Result<usize, Vec<ArtFailure>> {
    let art = farm_art(install, tileset).map_err(|f| vec![f])?;
    let path = resolve(install.root(), &art.wheat)
        .ok_or_else(|| vec![ArtFailure::Missing(art.wheat.clone())])?;
    let bytes = std::fs::read(path).map_err(|_| vec![ArtFailure::Missing(art.wheat.clone())])?;
    render::tga::decode(&bytes)
        .map(|_| 1)
        .map_err(|e| vec![ArtFailure::Undecodable(art.wheat.clone(), e.to_string())])
}

/// Every `.bh3` and `.bha` under `art/` and `terrain art/`, by path —
/// the denominator nobody chose (`docs/RENDERER.md` §5.2).
pub fn art_files(install: &Install, ext: &str) -> Vec<PathBuf> {
    fn walk(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, ext, out);
            } else if p
                .extension()
                .and_then(|x| x.to_str())
                .is_some_and(|x| x.eq_ignore_ascii_case(ext))
            {
                out.push(p);
            }
        }
    }
    let mut out = Vec::new();
    for d in ["art", "terrain art"] {
        walk(&install.root().join(d), ext, &mut out);
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The release frame is `starttime / 67`, floored at 1** (item 542,
    /// `docs/COMBAT.md` §50.1): `GraphicEvents::init_unit_events@008e2520`
    /// at `008e296d`–`008e299f`. Every release a capture had measured
    /// before run127 gives the same frame under the old `× 3 / 200`, and
    /// the Trireme's three are the first that do not: run127 launches its
    /// rounds on frames 5, 9 and 17 of the swing, four and twelve apart.
    #[test]
    fn a_release_frame_is_starttime_over_sixty_seven() {
        for ms in [466, 1533, 333, 733, 1666, 666, 833, 1066, 1465, 1532] {
            assert_eq!(release_frame(ms), ms * 3 / 200, "{ms} ms, measured before");
        }
        assert_eq!(
            [400, 666, 1200].map(release_frame),
            [5, 9, 17],
            "run127's three rounds"
        );
        assert_eq!([0, 10, 66, 67].map(release_frame), [1, 1, 1, 1]);
    }

    /// **`<RESTRICTION>` binds by `GRAPH`, case aside** (item 595,
    /// `docs/COMBAT.md` §52). The Chariot (`TypeIndex` 195, `GRAPH
    /// CHARIOT`) has one node, 4, over −180..180, the row run145's `0/8`
    /// shoots through. `Mameluke` names a record whose `GRAPH` is
    /// `MAMELUKE`: `String::operator==@00a1f140` is `_wcsicmp`, so it binds,
    /// and a case-sensitive match would have lost it. `SuperBattleship`
    /// names no `GRAPH` at all and binds to nothing. Made to fail first
    /// with `==` in place of `eq_ignore_ascii_case`.
    /// Item 3329: the Dutch caravan's `GRAPH` is `ArmedCaravan` and its
    /// art is `ARMEDCARAVAN-…`; the original plays it (run963's `1/14`,
    /// pieces 10 and 12682 on block 1765), so the piece table holds both.
    #[test]
    fn a_unit_piece_binds_by_graph_whatever_its_case() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let units = inst.units().expect("unitrules.xml");
        let graphs: Vec<String> = units
            .records
            .iter()
            .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
            .collect();
        assert_eq!(
            graphs[60 - 0x32],
            "ArmedCaravan",
            "the case differs in the file"
        );
        let lengths = piece_lengths(&inst, &graphs);
        assert!(lengths.contains_key(&10), "CARADUTCH's age-0 piece");
        assert!(lengths.contains_key(&12682), "and its crew's");
    }

    fn graphs(inst: &Install) -> Vec<String> {
        inst.units()
            .expect("unitrules.xml")
            .records
            .iter()
            .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
            .collect()
    }

    /// **Every model and animation file the install ships is read whole
    /// but two** (item 4019, `docs/FORMATS.md` §3): 1,845 of 1,847 `.bh3`
    /// under `art/` and `terrain art/`, and 2,062 of 2,062 `.bha`. The two
    /// models carry a node whose vertex run passes the vertex array — the
    /// original `memcpy`s and never checks — and they are named here, so a
    /// third fails by name rather than by count.
    #[test]
    fn every_model_and_animation_the_install_ships_decodes_but_two() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let mut failed = Vec::new();
        let models = art_files(&inst, "bh3");
        for f in &models {
            if let Err(e) = render::bh3::decode(&std::fs::read(f).unwrap()) {
                let name = f.file_name().unwrap().to_string_lossy().to_string();
                failed.push((name, e.reason));
            }
        }
        let anims = art_files(&inst, "bha");
        let mut anim_failed = Vec::new();
        for f in &anims {
            if let Err(e) = render::bha::decode(&std::fs::read(f).unwrap()) {
                anim_failed.push(format!("{}: {e}", f.display()));
            }
        }
        assert_eq!((models.len(), anims.len()), (1847, 2062), "the files read");
        use render::chunk::Reason::RangePastVerts;
        assert_eq!(
            failed,
            [
                (
                    "citizen6_fem_Na.BH3".to_string(),
                    RangePastVerts {
                        start: 219,
                        num: 21,
                        verts: 237
                    }
                ),
                (
                    "granary_3kz.BH3".to_string(),
                    RangePastVerts {
                        start: 0,
                        num: 104,
                        verts: 58
                    }
                ),
            ]
        );
        assert!(anim_failed.is_empty(), "{anim_failed:#?}");
    }

    /// **The tileset's tint** (item 4212; `docs/RENDERER.md` §3.21):
    /// `early_autumn`'s `<LIGHTING>` reads (18, 18, 0), `Snowy`'s (0, 9,
    /// 38), and a tileset the file does not hold is refused.
    #[test]
    fn the_tileset_tint_is_read() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        assert_eq!(tileset_tint(&inst, "early_autumn").unwrap(), [18, 18, 0]);
        assert_eq!(tileset_tint(&inst, "Snowy").unwrap(), [0, 9, 38]);
        assert!(tileset_tint(&inst, "no such tileset").is_err());
    }

    /// **Every image the install ships is read whole** (item 4037,
    /// `docs/FORMATS.md`, the building pieces' section): 2,901 of 2,901 `.tga` under `art/`
    /// and `terrain art/`, each ending at its last pixel (283) or at a
    /// version-2 footer — types 2, 3 and 10, no colour map.
    #[test]
    fn every_image_the_install_ships_decodes() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let files = art_files(&inst, "tga");
        let mut kinds = BTreeMap::new();
        let mut failed = Vec::new();
        for f in &files {
            match render::tga::decode(&std::fs::read(f).unwrap()) {
                Ok(img) => *kinds.entry((img.kind, img.bits, img.footer)).or_insert(0) += 1,
                Err(e) => failed.push(format!("{}: {e}", f.display())),
            }
        }
        assert!(failed.is_empty(), "{failed:#?}");
        assert_eq!(
            kinds.into_iter().collect::<Vec<_>>(),
            [
                ((2, 24, true), 4),
                ((2, 32, false), 283),
                ((2, 32, true), 499),
                ((3, 8, true), 1),
                ((10, 24, true), 1),
                ((10, 32, true), 2113),
            ],
            "(type, bits, footer) → files"
        );
    }

    /// **The lobby's trees are the packet's 24, and every one decodes**
    /// (item 4080): `early_autumn`'s `treescale` 1.3, times 3, is run1559b's
    /// `tree_scale` `0x40799999`, and its `#EARLY_AUTUMN_TREE_MODELS0` holds
    /// the 24 models `RTreeData::tree_data` counts on that packet. Made to
    /// fail once with the `DEFAULT` section's path: `./art/tree_t_00.bh3`
    /// is not under `dirty/`.
    #[test]
    fn the_lobby_s_trees_are_the_packet_s_and_decode() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let art = tree_art(&inst, "early_autumn").expect("early_autumn's trees");
        assert_eq!((art.treescale * 3.0).to_bits(), 0x4079_9999);
        assert_eq!(art.models.len(), 24);
        assert_eq!(
            art.models[0],
            "./terrain art/tilesets/dirty/dtrees/tree_t_00.bh3"
        );
        assert_eq!(
            art.models[23],
            "./terrain art/tilesets/dirty/dtrees/tree_t_27.bh3"
        );
        for m in &art.models {
            let bytes = std::fs::read(resolve(inst.root(), m).expect(m)).unwrap();
            render::bh3::decode(&bytes).unwrap_or_else(|e| panic!("{m}: {e}"));
        }
        // The seven textures (item 4082), each a 32-bit image whose alpha
        // the leaves' test reads.
        assert_eq!(art.textures.len(), 7);
        assert_eq!(
            art.textures[0],
            "./terrain art/tilesets/dirty/dtrees/tree_T_bark.tga"
        );
        assert_eq!(
            art.textures[6],
            "./terrain art/tilesets/dirty/dtrees/tree_Tea_top_a.tga"
        );
        for t in &art.textures {
            let bytes = std::fs::read(resolve(inst.root(), t).expect(t)).unwrap();
            let image = render::tga::decode(&bytes).unwrap_or_else(|e| panic!("{t}: {e}"));
            assert_eq!(image.bits, 32, "{t}");
        }
        assert!(tree_art(&inst, "no such tileset").is_err());
    }

    /// **The farm's art reads whole on every tileset** (item 4037): the
    /// `<FARM>` element as `Farms::init@008d9310` reads it, and each
    /// `<TILESET>`'s `FARMTEX` image decoding — seventeen tilesets, three
    /// images. Made to fail once by reading the `<FARM>`'s own `texture=`
    /// in the tileset's place: it names `.\art\farm_wheat_1.tga` too, so
    /// the snowy and jungle tilesets parted, which is why the reader
    /// follows `TileSet::init@0087b370`, not the attribute.
    #[test]
    fn the_farm_s_art_reads_whole_on_every_tileset() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let text = std::fs::read_to_string(inst.data("tilesets.xml")).unwrap();
        let doc = roxmltree::Document::parse(&text).unwrap();
        let mut wheat = BTreeMap::new();
        for t in doc.descendants().filter(|n| n.has_tag_name("TILESET")) {
            let name = t.attribute("name").unwrap();
            let art = farm_art(&inst, name).unwrap();
            assert_eq!(
                (art.stalk, art.heights, art.unread_texture.as_str()),
                ([166, 216, 188, 173], [10, 36], ".\\art\\farm_wheat_1.tga"),
                "{name}"
            );
            assert_eq!(farm_decoding(&inst, name), Ok(1), "{name}");
            *wheat.entry(art.wheat).or_insert(0) += 1;
        }
        assert_eq!(
            wheat.into_iter().collect::<Vec<_>>(),
            [
                ("./art/farm_jungle_wheat_1.tga".to_string(), 3),
                ("./art/farm_snowy_wheat_1.tga".to_string(), 2),
                ("./art/farm_wheat_1.tga".to_string(), 12),
            ]
        );
        assert!(matches!(
            farm_art(&inst, "no such tileset"),
            Err(ArtFailure::Unnamed(_))
        ));
    }

    /// **The whole reader and the simulation's agree on every animation**:
    /// `render::bha`'s first root's times are `key_times`' — which is
    /// diff-backed against every dumped `end_time` — on all 2,062 files.
    #[test]
    fn the_whole_animation_reader_agrees_with_key_times() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let files = art_files(&inst, "bha");
        assert_eq!(files.len(), 2062);
        for f in &files {
            let b = std::fs::read(f).unwrap();
            let whole = render::bha::decode(&b).expect("every .bha decodes");
            assert_eq!(
                Some(whole.nodes[0].times()),
                key_times(&b),
                "{}",
                f.display()
            );
        }
    }

    /// **Pieces decoding, K of N** — the renderer's first score
    /// ([`PIECES_DECODING`]). Every failing file is named with the pieces
    /// it costs, so a change says which file moved.
    #[test]
    fn the_pieces_decode() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let d = piece_decoding(&inst, &graphs(&inst));
        let failures: Vec<(String, usize)> = d
            .failures
            .iter()
            .map(|(k, (_, n))| (k.clone(), *n))
            .collect();
        eprintln!(
            "pieces decoding {} of {}; {} of {} entries are pieces; same shape {}; {failures:?}",
            d.decoded, d.pieces, d.entries_with_pieces, d.entries, d.same_shape
        );
        assert_eq!((d.decoded, d.pieces), PIECES_DECODING);
        assert_eq!(
            failures,
            [
                (".\\art\\armsuplytruck_attackwalk.bha".to_string(), 1),
                (".\\art\\ceo_crew3_idle3.bha".to_string(), 10),
                (".\\art\\citizen6_fem_NA.bh3".to_string(), 1),
                (".\\art\\siegeship_ship_walk.bha".to_string(), 3),
            ]
        );
        assert!(matches!(
            d.failures[".\\art\\ceo_crew3_idle3.bha"].0,
            ArtFailure::Missing(_)
        ));
        assert_eq!(d.same_shape, SAME_SHAPE);
    }

    /// **A gaia entry is `first_bird_piece + 3·(TypeIndex − 0x192) + v`**
    /// (item 4028, `unit_node_from_gpiece@00901c30`'s name): the sheep's
    /// `-TYPE2` is 60065 whatever its case, a variant past 2 is no piece, and
    /// a player unit's `GRAPH` with a `-TYPE` tail is not a gaia piece.
    #[test]
    fn a_gaia_entry_s_piece_is_the_bird_range_s() {
        let mut graphs = vec![String::new(); 0x19e - 0x32];
        graphs[0x198 - 0x32] = "HERDSHEEP".into();
        graphs[0x19b - 0x32] = "HERDFISH".into();
        graphs[0] = "SCOUT".into();
        assert_eq!(bird_pieces("HerdSheep-TYPE2", &graphs), Some(vec![60065]));
        assert_eq!(bird_pieces("HERDFISH-TYPE0", &graphs), Some(vec![60072]));
        assert_eq!(bird_pieces("HERDSHEEP-TYPE3", &graphs), None);
        assert_eq!(bird_pieces("SCOUT-TYPE0", &graphs), None);
        assert_eq!(FIRST_BIRD_PIECE, 60045);
    }

    /// **A building piece reads back to its `<BUILD>` name** (item 4028,
    /// `build_node_from_gpiece@008fd550`): run1428's capital `0/2000`
    /// prints piece 57144, which is age 5, style 5 (`ARAB`), record 0.
    #[test]
    fn a_building_piece_names_its_build_entry() {
        let graphs: Vec<String> = ["VILLAGE", "CITY", "METROPOLIS", "FARM"]
            .map(String::from)
            .to_vec();
        assert_eq!(
            build_piece_name(57144, &graphs),
            Some((414, "VILLAGE-ARAB-AGE5".into()))
        );
        assert_eq!(
            build_piece_name(FIRST_BUILD_PIECE + 3, &graphs),
            Some((417, "FARM-DEFAULT-AGE0".into()))
        );
        assert_eq!(
            build_piece_name(51212, &graphs),
            Some((416, "METROPOLIS-AMERICAN-AGE0".into()))
        );
        assert_eq!(build_piece_name(FIRST_BUILD_PIECE - 1, &graphs), None);
        assert_eq!(
            build_piece_name(FIRST_BUILD_PIECE + 8 * BUILD_PIECES_PER_AGE, &graphs),
            None
        );
        assert_eq!(BUILD_PIECES_PER_AGE, 9 * NUM_BUILD_PIECES);
    }

    /// **A building's addon subject is matched without case** (item 4028):
    /// `VILLAGE-ARAB-AGE5`'s `EXISTEVENT`s cite `metro_6t_ADDON`, which
    /// `effects_graphics.xml` writes `metro_6t_addon`; the loader's
    /// `String::operator==@00a1f140` falls to `_wcsicmp`, so the addon has a
    /// model, and every file the piece names decodes.
    #[test]
    fn a_building_s_addon_subject_matches_without_case() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let pieces = std::collections::BTreeSet::from([57144, FIRST_BUILD_PIECE + 3]);
        let art = build_art(&inst, &pieces);
        let village = art[&57144].as_ref().unwrap();
        eprintln!("{village:?}");
        assert_eq!(village.zbuffer.as_deref(), Some(".\\art\\village_6tz.bh3"));
        assert_eq!(village.texture, ".\\art\\village_6t.tga");
        assert_eq!(village.sprite_offset, (290, 247), "spritexoff, spriteyoff");
        assert_eq!(village.addons.len(), 2);
        assert!(
            village
                .addons
                .iter()
                .all(|(s, m, _, f)| s == "metro_6t_ADDON" && m.is_some() && f.is_some())
        );
        let farm = art[&(FIRST_BUILD_PIECE + 3)].as_ref().unwrap();
        assert_eq!((farm.zbuffer.as_ref(), farm.addons.len()), (None, 0));
        assert_eq!(0x19e + 3, FARM_TYPE);
        let d = building_decoding(&inst, &pieces, "Dirty");
        assert_eq!(
            d[&57144],
            Ok(5),
            "a zbuffer, two addon models, two animations"
        );
        // The farm names no model; it is held to its wheat image instead
        // (item 4037; `Ok(0)` before it).
        assert_eq!(
            d[&(FIRST_BUILD_PIECE + 3)],
            Ok(1),
            "the farm's one file, the tileset's wheat"
        );
    }

    /// **Every turn slot the install ships names a looping file** (item
    /// 3735, `docs/AI.md` §364.1). `sim::units::anim::non_looping` decides by
    /// slot, and calls `CHAR_TURN_LEFT`/`CHAR_TURN_RIGHT` looping because
    /// all 546 of `unit_graphics.xml`'s turn rows resolve to an `<ANIM>`
    /// under `<LOOPING>` — the packed Trebuchet's to `TrebuchetWagon
    /// Walk`. A turn row naming a non-looping file would make the rule
    /// wrong for that piece, and this is where it would show.
    #[test]
    fn every_turn_slot_names_a_looping_file() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let apath = inst.data("anim_graphics.xml");
        let upath = inst.data("unit_graphics.xml");
        let atext = crate::read(&apath).expect("anim_graphics.xml");
        let utext = crate::read(&upath).expect("unit_graphics.xml");
        let adoc = crate::parse(&apath, &atext).expect("anim_graphics.xml parses");
        let udoc = crate::parse(&upath, &utext).expect("unit_graphics.xml parses");
        let files = anim_files(&adoc);
        let mut turns = 0;
        let mut not_looping = Vec::new();
        for (unit, rows) in unit_anims(&udoc) {
            for (slot, anim) in rows {
                if slot != sim::units::anim::TURN_LEFT && slot != sim::units::anim::TURN_RIGHT {
                    continue;
                }
                turns += 1;
                if !files.get(&anim_key(&anim)).is_some_and(|(_, l)| *l) {
                    not_looping.push(format!("{unit} {slot} {anim}"));
                }
            }
        }
        assert_eq!(turns, 546, "the shipped turn rows");
        assert!(not_looping.is_empty(), "{not_looping:?}");
        assert!(
            !sim::units::anim::non_looping(sim::units::anim::TURN_LEFT)
                && !sim::units::anim::non_looping(sim::units::anim::TURN_RIGHT),
            "the slot rule agrees with the install"
        );
        let packed = unit_anims(&udoc);
        let treb = &packed["TREBUCHET-DEFAULT-AGE0-PACKED"];
        assert!(
            treb.contains(&(
                sim::units::anim::TURN_LEFT,
                "TrebuchetWagon Walk".to_string()
            )),
            "the packed Trebuchet turns on its wagon's walk"
        );
    }

    #[test]
    fn a_pivot_restriction_binds_by_graph_whatever_its_case() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let units = inst.units().expect("unitrules.xml");
        let graphs: Vec<String> = units
            .records
            .iter()
            .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
            .collect();
        let pivots = pivot_restrictions(&inst, &graphs);
        assert_eq!(
            pivots.get(&195),
            Some(&[(4, (-180, 180))].into_iter().collect())
        );
        let mameluke = graphs
            .iter()
            .position(|g| g.eq_ignore_ascii_case("Mameluke"))
            .expect("a Mameluke graph");
        assert_ne!(graphs[mameluke], "Mameluke", "the case differs in the file");
        assert_eq!(
            pivots.get(&(0x32 + mameluke as i32)),
            Some(&[(4, (-150, 150))].into_iter().collect())
        );
        assert!(
            !graphs
                .iter()
                .any(|g| g.eq_ignore_ascii_case("SuperBattleship"))
        );
        let rows: usize = pivots.values().map(|n| n.len()).sum();
        assert!(rows > 60, "most of the 81 rows bind: {rows}");
    }

    /// **Every release event is a round, and a `do_damage="0"` one is
    /// harmless** (item 853): `FIGHTER` fires eight frames on each of two
    /// nodes, all damaging; `FIGHTERBOMBER` the same sixteen with six
    /// `NoDamage Tracer`s among them, so ten damage a swing as in the
    /// original. Made to fail with the flag read as always false.
    #[test]
    fn a_release_keeps_its_node_and_its_ammo_s_damage() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let units = inst.units().expect("unitrules.xml");
        let graphs: Vec<String> = units
            .records
            .iter()
            .map(|r| r.text("GRAPH").unwrap_or_default().trim().to_string())
            .collect();
        let quiet = harmless_ammo(&inst);
        assert_eq!(quiet.len(), 3, "{quiet:?}");
        let rel = piece_releases(&inst, &graphs);
        let piece = |g: &str| {
            let ty = 0x32
                + graphs
                    .iter()
                    .position(|x| x.eq_ignore_ascii_case(g))
                    .expect("a graph") as i32;
            PieceName::parse(&format!("{g}-DEFAULT-AGE0"))
                .unwrap()
                .piece(ty)
        };
        let swing = |g: &str| rel[&piece(g)][&sim::units::anim::ATTACK2].clone();
        let fighter = swing("FIGHTER");
        assert_eq!(fighter.len(), 16);
        assert_eq!(fighter.iter().filter(|r| r.1 == 1).count(), 8);
        assert!(fighter.iter().all(|r| !r.2));
        let fb = swing("FIGHTERBOMBER");
        assert_eq!(fb.len(), 16);
        assert_eq!(fb.iter().filter(|r| !r.2).count(), 10, "ten damage");
        assert!(
            fighter.iter().chain(&fb).all(|r| !r.3),
            "guns, not missiles"
        );
        // The Advanced Battleship's six cruise missiles (item 1625):
        // `Cruisemissile0`/`1`, `missile="1"`, frames 19..23.
        let missiles = missile_ammo(&inst);
        assert_eq!(missiles.len(), 10, "{missiles:?}");
        let ship = rel[&piece("ADVANCEDBATTLESHIP")][&sim::units::anim::ATTACK1].clone();
        assert_eq!(
            ship.iter().map(|r| (r.0, r.3)).collect::<Vec<_>>(),
            [
                (19, true),
                (20, true),
                (21, true),
                (22, true),
                (23, true),
                (23, true)
            ]
        );
        // The burning rounds (item 3580): five `flame="1"` ammos, and the
        // Fire Lances' piece 46 (run1220's `GUY gpiece`) releases one on
        // each of its three attacks; no gun burns.
        let burning = flame_ammo(&inst);
        assert_eq!(burning.len(), 5, "{burning:?}");
        assert!(burning.contains("FirelanceAmmo"));
        assert_eq!(piece("FIRELANCES"), 46);
        let lances = &rel[&46];
        assert_eq!(
            [
                sim::units::anim::ATTACK1,
                sim::units::anim::ATTACK2,
                sim::units::anim::ATTACK3
            ]
            .iter()
            .map(|a| lances[a].iter().map(|r| r.4).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
            [[true], [true], [true]]
        );
        assert!(fighter.iter().chain(&fb).chain(&ship).all(|r| !r.4));
    }

    /// The `<UNIT>` name grammar, and the piece each coordinate lands on.
    /// `GraphicPieces::init_piece_ranges@008f70e0`'s four strides nest —
    /// six art styles to an age, three ages to a gender, two genders to a
    /// crew — and `total_num_unit_pieces` is `0xc606`, four crews plus the
    /// six "over time" pieces. Player 0's scout on run12 is `371` and its
    /// dog `13043`, which is this arithmetic.
    #[test]
    fn a_unit_graphic_s_name_gives_its_piece() {
        let p = PieceName::parse("SCOUT-ARAB-AGE0").expect("a unit entry");
        assert_eq!(
            (p.graph, p.style, p.age, p.gender, p.crew),
            ("SCOUT", 1, 0, 0, 0)
        );
        assert_eq!(p.piece(69), 371);
        let d = PieceName::parse("SCOUT-ARAB-AGE0-CREW1").expect("the dog");
        assert_eq!(d.crew, 1);
        assert_eq!(d.piece(69), 13043);
        // `AGE3` and `AGE5` are the ages themselves, folded to the two
        // upper brackets by `age < 5 ? age / 3 : 2`.
        assert_eq!(PieceName::parse("CITIZENS-DEFAULT-AGE3").unwrap().age, 1);
        assert_eq!(PieceName::parse("CITIZENS-DEFAULT-AGE5").unwrap().age, 2);
        // `-FEMALE` and `-PACKED` are the same coordinate.
        assert_eq!(
            PieceName::parse("CITIZENS-ARAB-AGE0-FEMALE")
                .unwrap()
                .piece(50),
            6688
        );
        assert_eq!(
            PieceName::parse("CATAPULT-DEFAULT-AGE0-PACKED")
                .unwrap()
                .gender,
            1
        );
        // The four crews and the six styles bound the pool.
        assert!(
            PieceName::parse("SCOUT-DEFAULT-AGE5-CREW3")
                .unwrap()
                .piece(0x191)
                < 0xc606
        );
        // A gaia entry, and a style the unit path cannot build, are not
        // pieces at all.
        assert!(PieceName::parse("HERDSHEEP-TYPE1").is_none());
        assert!(PieceName::parse("MERCHANT-KOREAN-AGE3").is_none());
        assert!(PieceName::parse("NOTAUNITNAME").is_none());
    }

    /// The header arithmetic, hand-built: two keys a third of a second
    /// apart is 333 ms and then 666, which is ten frames.
    #[test]
    fn a_two_key_node_reads_its_times_and_rounds_to_frames() {
        let mut b = vec![0u8; 28 + 2 * 36];
        b[16..20].copy_from_slice(&((28 + 2 * 36 - 16) as u32).to_le_bytes());
        b[24..28].copy_from_slice(&2u32.to_le_bytes());
        b[28..32].copy_from_slice(&(1.0f32 / 3.0).to_bits().to_le_bytes());
        b[64..68].copy_from_slice(&(1.0f32 / 3.0).to_bits().to_le_bytes());
        let t = key_times(&b).expect("two keys");
        assert_eq!(t, vec![333, 666]);
        assert_eq!(game_frames(&t, true), 10);
        // The same node, non-looping: the last key is dropped and 333 ms
        // is five frames.
        assert_eq!(game_frames(&t, false), 5);
    }

    /// A chunk whose key count does not fill it is not a node this reader
    /// understands, and it says so rather than reading past the end.
    #[test]
    fn a_chunk_whose_keys_do_not_fill_it_is_refused() {
        let mut b = vec![0u8; 28 + 2 * 36];
        b[16..20].copy_from_slice(&64u32.to_le_bytes());
        b[24..28].copy_from_slice(&2u32.to_le_bytes());
        assert!(key_times(&b).is_none());
    }

    /// `round(times · 3 / 200)` on the half: 6000 ms is 90 frames, and
    /// 6033 is still 90 because the third of a frame rounds down.
    #[test]
    fn the_frame_count_rounds_half_up() {
        assert_eq!(game_frames(&[6000], true), 90);
        assert_eq!(game_frames(&[6033], true), 90);
        assert_eq!(game_frames(&[6066], true), 91);
        assert_eq!(game_frames(&[1056], true), 16);
        assert_eq!(game_frames(&[1023], true), 15);
    }

    /// `force_load`'s non-looping arm, on the shipped numbers that made it
    /// visible: `lumberjack_dump.bha` ends 2157, 2190 and its slot is 32
    /// frames rather than 33 — which is what every `GUY` block in the
    /// corpus prints for `cur_anim 27`. The guards are the original's own:
    /// a last key of zero is left alone, and a single-key node has no
    /// second-to-last to take.
    #[test]
    fn a_non_looping_animation_drops_its_last_key() {
        assert_eq!(game_frames(&[2157, 2190], true), 33);
        assert_eq!(game_frames(&[2157, 2190], false), 32);
        assert_eq!(game_frames(&[900, 0], false), 0);
        assert_eq!(game_frames(&[2190], false), 33);
    }

    /// **The foundation's and the scaffold's own pieces decode** (item
    /// 4163, `docs/RENDERER.md` §3.5): each `<CONSTRUCTION>`'s `texture=`
    /// under `render::tga` (and the dock's `model=` under `render::bh3`),
    /// and each `<SCAFFOLD>`'s `model=`, `texture=` and its `anim=`'s
    /// `.BHa` — what the client draws them from. Made to fail: a texture
    /// read as a model fails every entry.
    #[test]
    fn the_foundation_s_and_the_scaffold_s_pieces_decode() {
        let Some(inst) = crate::testenv::install() else {
            eprintln!("skipping: no install");
            return;
        };
        let read = |rel: &str| resolve(inst.root(), rel).and_then(|p| std::fs::read(p).ok());
        let tga = |rel: &str| read(rel).is_some_and(|b| render::tga::decode(&b).is_ok());
        let bh3 = |rel: &str| read(rel).is_some_and(|b| render::bh3::decode(&b).is_ok());
        let bha = |rel: &str| read(rel).is_some_and(|b| render::bha::decode(&b).is_ok());
        let found = construction_art(&inst);
        let ok = found
            .iter()
            .filter(|c| tga(&c.texture) && c.model.as_deref().is_none_or(bh3))
            .count();
        assert_eq!((ok, found.len()), CONSTRUCTION_DECODING, "{found:?}");
        assert_eq!(
            found[9].model.as_deref(),
            Some(".\\art\\construction_dock.bh3")
        );
        let scaffolds = scaffold_art(&inst);
        let ok = scaffolds
            .iter()
            .filter(|s| {
                bh3(&s.model)
                    && tga(&s.texture)
                    && s.anim
                        .as_ref()
                        .is_none_or(|(_, f)| f.as_deref().is_some_and(bha))
            })
            .count();
        assert_eq!((ok, scaffolds.len()), SCAFFOLD_DECODING, "{scaffolds:?}");
        // The elements `scaffold_timing` numbers are this walk's.
        let names: Vec<String> = scaffolds
            .iter()
            .map(|s| s.name.to_ascii_lowercase())
            .collect();
        for t in &scaffold_timing(&inst).unwrap().tables {
            for slot in t {
                let first = names
                    .iter()
                    .position(|n| *n == names[usize::from(slot.element)]);
                assert_eq!(first, Some(usize::from(slot.element)));
            }
        }
    }
}
