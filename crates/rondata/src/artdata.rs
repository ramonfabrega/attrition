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
//!   · 3 / 200)` — fifteen frames a second.
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

/// The `UnitAnim` slot each `<ANIM name="CHAR_…">` names. The indices are
/// `sim::anim`'s, which are the enum's (`rise.pdb`, type 0x46B1); a name
/// this table does not carry is skipped rather than guessed.
const SLOTS: &[(&str, i8)] = &[
    ("CHAR_DEFAULT", sim::anim::DEFAULT),
    ("CHAR_IDLE1", sim::anim::IDLE1),
    ("CHAR_IDLE2", sim::anim::IDLE2),
    ("CHAR_IDLE3", sim::anim::IDLE3),
    ("CHAR_SLOG", sim::anim::SLOG),
    ("CHAR_WALK", sim::anim::WALK),
    ("CHAR_JOG", sim::anim::JOG),
    ("CHAR_ATTACKWALK", sim::anim::ATTACKWALK),
    ("CHAR_ATTACK1", sim::anim::ATTACK1),
    ("CHAR_ATTACK2", sim::anim::ATTACK2),
    ("CHAR_ATTACK3", sim::anim::ATTACK3),
    ("CHAR_TURN_LEFT", sim::anim::TURN_LEFT),
    ("CHAR_TURN_RIGHT", sim::anim::TURN_RIGHT),
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
/// `docs/SYNC.md` §3.6 — are **left out on purpose**: they name a `-TYPE0`
/// and a `-TYPE1` and no `-TYPE2`, so a third of them would be a guess,
/// and giving the other two lengths would start their idle rolls drawing
/// in the same commit as the bird's. They are the queue's, not this
/// item's; without a row here they keep the dump's table and the
/// behaviour they had.
const GAIA_UNITS: &[(i32, &str)] = &[
    (0x192, "WILDBIRD"),
    (0x193, "FLOCKBIRD"),
    (0x194, "GULLBIRD"),
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
                let Some(file) = files.get(anim) else {
                    continue;
                };
                let n = match frames.get(file) {
                    Some(&n) => n,
                    None => {
                        let Some(n) = file_frames(install.root(), file) else {
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

/// `anim_graphics.xml`'s `<ANIM name= file=>`: the animation's name as a
/// `<UNIT>` cites it, and the file it resolves to (`.\art\x.bha` kept as
/// the install-relative path it is).
fn anim_files(doc: &roxmltree::Document<'_>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for n in doc.descendants().filter(|n| n.has_tag_name("ANIM")) {
        let (Some(name), Some(file)) = (n.attribute("name"), n.attribute("file")) else {
            continue;
        };
        out.insert(name.trim().to_string(), file.trim().to_string());
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
        if !rows.is_empty() {
            out.insert(name.trim().to_string(), rows);
        }
    }
    out
}

/// `AnimMgr::force_load`'s `frames[]` for one animation file: the root
/// node's last key time in milliseconds, at fifteen frames a second.
pub fn file_frames(root: &Path, rel: &str) -> Option<u32> {
    let bytes = std::fs::read(resolve(root, rel)?).ok()?;
    Some(game_frames(&key_times(&bytes)?))
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
pub fn key_times(bytes: &[u8]) -> Option<Vec<u16>> {
    let word = |at: usize| -> Option<u32> {
        bytes
            .get(at..at + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    let size = word(16)? as usize;
    let n = word(24)? as usize;
    // The node's own arithmetic, which is what says the offsets are right:
    // the chunk ends exactly where its keys do.
    if 28 + n.checked_mul(36)? != 16 + size || n == 0 {
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

/// `AnimMgr::force_load`'s `frames[] = round(times · 3 / 200)` — the whole
/// of the conversion from the file's milliseconds to the animation clock's
/// frames, and the reason a length is what it is. Fifteen frames a second,
/// rounded half up.
pub fn game_frames(times: &[u16]) -> u32 {
    let ms = u32::from(*times.last().unwrap_or(&0));
    // `(int)(ms · 3 / 200.0f)`, then `+1` when the remainder is at least a
    // half — integer arithmetic for the same answer.
    let n = ms * 3;
    n / 200 + u32::from(n % 200 * 2 >= 200)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(game_frames(&t), 10);
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
        assert_eq!(game_frames(&[6000]), 90);
        assert_eq!(game_frames(&[6033]), 90);
        assert_eq!(game_frames(&[6066]), 91);
        assert_eq!(game_frames(&[1056]), 16);
        assert_eq!(game_frames(&[1023]), 15);
    }
}
