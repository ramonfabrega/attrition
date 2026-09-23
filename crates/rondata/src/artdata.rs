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
/// `sim::anim`'s, which are the enum's (`rise.pdb`, type 0x46B1); a name
/// this table does not carry is skipped rather than guessed.
const SLOTS: &[(&str, i8)] = &[
    ("CHAR_DEFAULT", sim::anim::DEFAULT),
    ("CHAR_IDLE1", sim::anim::IDLE1),
    ("CHAR_IDLE2", sim::anim::IDLE2),
    ("CHAR_IDLE3", sim::anim::IDLE3),
    ("CHAR_GROUP_IDLE1", sim::anim::GROUP_IDLE1),
    ("CHAR_GROUP_IDLE2", sim::anim::GROUP_IDLE2),
    ("CHAR_GROUP_IDLE3", sim::anim::GROUP_IDLE3),
    ("CHAR_SLOG", sim::anim::SLOG),
    ("CHAR_WALK", sim::anim::WALK),
    ("CHAR_JOG", sim::anim::JOG),
    ("CHAR_ATTACKWALK", sim::anim::ATTACKWALK),
    ("CHAR_ATTACK1", sim::anim::ATTACK1),
    ("CHAR_ATTACK2", sim::anim::ATTACK2),
    ("CHAR_ATTACK3", sim::anim::ATTACK3),
    ("CHAR_ATTACKSPECIAL", sim::anim::ATTACKSPECIAL),
    ("CHAR_DEATH_STAB1", sim::anim::DEATH_STAB1),
    ("CHAR_DEATH_STAB2", sim::anim::DEATH_STAB2),
    ("CHAR_DEATH_SHOT1", sim::anim::DEATH_SHOT1),
    ("CHAR_DEATH_SHOT2", sim::anim::DEATH_SHOT2),
    ("CHAR_DEATH_SPLODED1", sim::anim::DEATH_SPLODED1),
    ("CHAR_DEATH_SPLODED2", sim::anim::DEATH_SPLODED2),
    ("CHAR_TURN_LEFT", sim::anim::TURN_LEFT),
    ("CHAR_TURN_RIGHT", sim::anim::TURN_RIGHT),
    ("CHAR_PACK", sim::anim::PACK),
    ("CHAR_UNPACK", sim::anim::UNPACK),
    ("CHAR_CHOP_WOOD", sim::anim::CHOP_WOOD),
    ("CHAR_WALK_WITH_WOOD", sim::anim::WALK_WITH_WOOD),
    ("CHAR_DUMP_WOOD", sim::anim::DUMP_WOOD),
    ("CHAR_WALK_TO_WOOD", sim::anim::WALK_TO_WOOD),
    ("CHAR_MINE_ORE", sim::anim::MINE_ORE),
    ("CHAR_WALK_WITH_ORE", sim::anim::WALK_WITH_ORE),
    ("CHAR_DUMP_ORE", sim::anim::DUMP_ORE),
    ("CHAR_WALK_TO_ORE", sim::anim::WALK_TO_ORE),
    ("CHAR_BUILD", sim::anim::BUILD),
    ("CHAR_REPAIR", sim::anim::REPAIR),
    ("CHAR_SOW", sim::anim::SOW),
    ("CHAR_REAP", sim::anim::REAP),
    ("CHAR_FARM", sim::anim::FARM),
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
/// frames (`sim::anim::MISSING`). Returns an empty map when either XML is
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
                let Some((file, looping)) = files.get(anim) else {
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
/// three-frame fallback ([`sim::anim::MISSING`]). Returns an empty map
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
    // `GRAPH` to the `TypeIndex`es that play it. The gaia records
    // (`0x192` up) are left out: their pieces come from
    // `first_bird_piece`, not from this arithmetic ([`gaia_lengths`]).
    let mut by_graph: BTreeMap<&str, Vec<i32>> = BTreeMap::new();
    for (i, g) in graphs.iter().enumerate() {
        let ty = 0x32 + i as i32;
        if ty >= 0x192 {
            break;
        }
        by_graph.entry(g.trim()).or_default().push(ty);
    }
    let mut frames: BTreeMap<&str, u32> = BTreeMap::new();
    let mut out = PieceLengths::new();
    for (name, rows) in &units {
        let Some(p) = PieceName::parse(name) else {
            continue;
        };
        let Some(types) = by_graph.get(p.graph) else {
            continue;
        };
        let mut lengths: BTreeMap<i8, u32> = BTreeMap::new();
        for (slot, anim) in rows {
            let Some((file, looping)) = files.get(anim) else {
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
        for &ty in types {
            out.insert(p.piece(ty), lengths.clone());
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
pub type PieceReleases = BTreeMap<i32, BTreeMap<i8, Vec<u32>>>;

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

/// Every player unit piece's [`PieceReleases`] entry, read from the
/// install.
///
/// `graphs` is the same `GRAPH` column [`piece_lengths`] takes and the
/// name walk is [`piece_tracks`]'s. Only the rows whose `anim` names a
/// slot this crate knows are kept, and the frames of one slot come back
/// **sorted and deduplicated** — the file writes them in order already,
/// but the event walk's `last_time < t <= cur_time` test does not care
/// and a stable order is what makes the draw sequence reproducible.
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
    let mut by_graph: BTreeMap<&str, Vec<i32>> = BTreeMap::new();
    for (i, g) in graphs.iter().enumerate() {
        let ty = 0x32 + i as i32;
        if ty >= 0x192 {
            break;
        }
        by_graph.entry(g.trim()).or_default().push(ty);
    }
    let mut out = PieceReleases::new();
    for u in udoc.descendants().filter(|n| n.has_tag_name("UNIT")) {
        let Some(name) = u.attribute("name") else {
            continue;
        };
        let Some(p) = PieceName::parse(name.trim()) else {
            continue;
        };
        let Some(types) = by_graph.get(p.graph) else {
            continue;
        };
        let mut rows: BTreeMap<i8, Vec<u32>> = BTreeMap::new();
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
            rows.entry(slot).or_default().push(release_frame(ms));
        }
        if rows.is_empty() {
            continue;
        }
        for v in rows.values_mut() {
            v.sort_unstable();
            v.dedup();
        }
        for &ty in types {
            out.insert(p.piece(ty), rows.clone());
        }
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
    let mut by_graph: BTreeMap<&str, Vec<i32>> = BTreeMap::new();
    for (i, g) in graphs.iter().enumerate() {
        let ty = 0x32 + i as i32;
        if ty >= 0x192 {
            break;
        }
        by_graph.entry(g.trim()).or_default().push(ty);
    }
    let mut out = PieceTracks::new();
    for u in udoc.descendants().filter(|n| n.has_tag_name("UNIT")) {
        let Some(name) = u.attribute("name") else {
            continue;
        };
        let Some(p) = PieceName::parse(name.trim()) else {
            continue;
        };
        let Some(types) = by_graph.get(p.graph) else {
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
        for &ty in types {
            out.insert(p.piece(ty), (dx, dy));
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
fn anim_files(doc: &roxmltree::Document<'_>) -> BTreeMap<String, (String, bool)> {
    let mut out = BTreeMap::new();
    for n in doc.descendants().filter(|n| n.has_tag_name("ANIM")) {
        let (Some(name), Some(file)) = (n.attribute("name"), n.attribute("file")) else {
            continue;
        };
        let looping = n.parent().is_some_and(|p| p.has_tag_name("LOOPING"));
        out.insert(name.trim().to_string(), (file.trim().to_string(), looping));
    }
    out
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
}
