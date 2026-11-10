//! **The map's trees, stood where the original stands them** (item 4080;
//! `docs/RENDERER.md` §3.7).
//!
//! The simulation keeps which cells are forest (`WData.flags & 0x20`) and
//! nothing else about a tree: where each one stands in its cell, which model
//! it is and its size are the renderer's, decided by
//! `ObjectsOut::sort_trees@00656c70` each time the fog or the camera moves.
//! For every visible forest cell it takes the neighbour mask
//! (`ObjectsData::make_tree_mask@00658880`), places five trees
//! (`ObjectsData::place_tree@00658530`) about the cell's centre, and draws
//! each (`GraphicPieces::draw_tree@00905a10`) as the tileset's model
//! `(int)(y + x) % models`, scaled by the tileset's `treescale` × 3 with y
//! negated, at the place.
//!
//! **The generator is the cell's own.** `place_tree` seeds a local `Random`
//! with `(ulong)(cx · 7.0 + cy)` at the cell's centre on every call, so a
//! tree's place is a function of the map alone, never of the session or of
//! the game's generator. Its draws are `Random::get@00a39d70`'s LCG.
//!
//! Every value here is an integer the original holds in a single exactly:
//! the centre, the offsets and the draws are integers under 2²⁴ on any map
//! under 2,700 cells across, so the singles' sums are exact.

use crate::terrain::Heightfield;

/// A cell's side in world units: `sort_trees`' `x · 0x300 + 0x180`.
pub const CELL: i32 = 0x300;

/// The `<TREES treescale=…>` factor `TileSet::init@0087b370` multiplies by
/// (`× 3.0`, `__real_40400000`) into `TileSetData::tree_scale`, which
/// `TileSetData::load_all@008798b0` copies to the global `draw_tree` reads.
pub const TREESCALE_FACTOR: f32 = 3.0;

/// The slots `sort_trees` places in a 3D-tree cell (`section_render[10]`
/// set): all five. With sprite trees it skips slot 2.
pub const SLOTS: i32 = 5;

/// `Random::get@00a39d70`: the LCG `s · 0x19660d + 0x3c6ef35f`, and
/// `((s & 0xffff) · (hi − lo) >> 16) + lo`; equal bounds return the bound
/// and leave the seed alone.
#[derive(Clone, Copy, Debug)]
pub struct Random(pub u32);

impl Random {
    pub fn get(&mut self, a: i32, b: i32) -> i32 {
        if a == b {
            return a;
        }
        let (lo, hi) = if b < a { (b, a) } else { (a, b) };
        self.0 = self.0.wrapping_mul(0x19660d).wrapping_add(0x3c6ef35f);
        (((self.0 & 0xffff).wrapping_mul((hi - lo) as u32)) >> 16) as i32 + lo
    }
}

/// `make_tree_mask`: bit 1 the cell west (x − 1), 2 north (y − 1), 4 east
/// (x + 1), 8 south (y + 1), each set when that neighbour is forest. Off
/// the map reads not forest.
pub fn tree_mask(forest: impl Fn(i32, i32) -> bool, x: i32, y: i32) -> u8 {
    [(-1, 0, 1), (0, -1, 2), (1, 0, 4), (0, 1, 8)]
        .iter()
        .filter(|&&(dx, dy, _)| forest(x + dx, y + dy))
        .fold(0, |m, &(_, _, b)| m | b)
}

/// `place_tree`'s place for `slot` in the cell `(cx, cy)` with neighbour
/// `mask`, in world units. A side whose neighbour is forest reaches 144
/// toward it, else 96. **Slot 0 has no `break`**: the listing's case 0
/// falls into case 1 (the jump table at `0x658864`; nothing jumps out at
/// `0x658632`), so slot 0 takes slot 1's offsets and draws too, on the same
/// generator — four draws to slot 1's two.
pub fn place(cx: i32, cy: i32, mask: u8, slot: i32) -> (i32, i32) {
    let reach = |bit: u8| if mask & bit != 0 { 144 } else { 96 };
    let (w, n, e, s) = (reach(1), reach(2), reach(4), reach(8));
    let (mut x, mut y) = (cx * CELL + CELL / 2, cy * CELL + CELL / 2);
    let mut r = Random((x * 7 + y) as u32);
    match slot {
        0 | 1 => {
            if slot == 0 {
                x -= w;
                y -= n;
                x += r.get(-w, 0);
                y += r.get(-n, 0);
            }
            x -= w;
            y += s;
            x += r.get(-w, 0);
            y += r.get(0, s);
        }
        2 => {
            x += r.get(-w, e);
            y += r.get(-n, s);
        }
        3 => {
            x += e;
            y -= n;
            x += r.get(0, e);
            y += r.get(-n, 0);
        }
        4 => {
            x += e;
            y += s;
            x += r.get(0, e);
            y += r.get(0, s);
        }
        _ => {}
    }
    (x, y)
}

/// `draw_tree`'s model: `cur_tree_index = (int)(y + x) % models`.
pub fn model(x: i32, y: i32, models: usize) -> usize {
    (x + y).rem_euclid(models as i32) as usize
}

/// `TerrainOut::find_data_z@00866560(x, y, 0)` over the door's corner
/// singles, in the listing's single arithmetic and order — the renderer's
/// copy of `sim::World::data_z`, which is the simulation's own.
pub fn data_z(field: &Heightfield<'_>, x: i32, y: i32) -> i32 {
    let (sx, _) = field.stride();
    if field.corners.is_empty() {
        return 0;
    }
    let inv = f32::from_bits(0x3baa_aaab);
    let (tw, th) = (field.width * 4, field.height * 4);
    let div3 = |v: i32| (v >> 6).max(0) / 3;
    let (ix, iy) = (div3(x).min(tw - 1), div3(y).min(th - 1));
    let i0 = (sx * iy + ix) as usize;
    let h = |i: usize| f32::from_bits(field.corners[i]);
    let row = sx as usize;
    let (fx, fy) = (x - ix * 0xc0, y - iy * 0xc0);
    let (h01, h10) = (h(i0 + row), h(i0 + 1));
    let term = |d: f32, f: i32| (d * inv) * f as f32;
    let (base, a, b) = if fx > 0xc0 - fy {
        let h11 = h(i0 + row + 1);
        (h10, term(h11 - h10, fy), term(h01 - h11, 0xc0 - fx))
    } else {
        let h00 = h(i0);
        (h00, term(h10 - h00, fx), term(h01 - h00, fy))
    };
    (((base + a) + 0.0) + b) as i32
}

/// **A tree's model-to-world transform**: `draw_tree` resets the tree
/// data's transform, sets its scale to `(s, −s, s)` — `tree_scale` and its
/// negation, `xorps` with the sign mask at `0x905ab4` — and its translation
/// to the place; no turn. A model point `m` lands at
/// `at + s·(m.x, −m.y, m.z)`.
pub fn tree_placement(at: (i32, i32, i32), scale: f32) -> crate::camera::Mat4 {
    let mut m = crate::camera::Mat4::IDENTITY;
    m.0[0] = scale;
    m.0[5] = -scale;
    m.0[10] = scale;
    m.0[12] = at.0 as f32;
    m.0[13] = at.1 as f32;
    m.0[14] = at.2 as f32;
    m
}

/// One tree as `sort_trees` stands it: its cell, mask and slot, its place
/// in world units and the model it draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tree {
    pub cell: (i32, i32),
    pub mask: u8,
    pub slot: i32,
    pub at: (i32, i32, i32),
    pub model: usize,
}

/// Every tree of the forest cells `cells` names, five a cell, in the
/// order given: `forest` is the map's `WData.flags & 0x20`.
pub fn stand(
    field: &Heightfield<'_>,
    forest: impl Fn(i32, i32) -> bool,
    cells: impl IntoIterator<Item = (i32, i32)>,
    models: usize,
) -> Vec<Tree> {
    let mut out = Vec::new();
    for (cx, cy) in cells {
        if !forest(cx, cy) {
            continue;
        }
        let mask = tree_mask(&forest, cx, cy);
        for slot in 0..SLOTS {
            let (x, y) = place(cx, cy, mask, slot);
            out.push(Tree {
                cell: (cx, cy),
                mask,
                slot,
                at: (x, y, data_z(field, x, y)),
                model: model(x, y, models),
            });
        }
    }
    out
}

/// One of a tree model's six node slots, as
/// `TreeTransformMethod::make_index_table@00920df0` fills `TreeIndexTable`:
/// the node's run of vertices and the texture (0–6) its section samples.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub start: u32,
    pub num: u32,
    pub texture: usize,
}

/// **A tree model's node slots** (item 4082, `docs/RENDERER.md` §3.7).
/// A node with vertices is named into a slot: `root` 0 and `Trunk` 1, both
/// the bark (texture 0); the name less its last two characters `Bottom_1`
/// 2, `Middle_1` 3, `Middle_2` 4 and `Top_1` 5. The last character picks
/// the leaves: a bottom's `A`/`B` textures 1/2, a middle's `A`/`B`/`C`
/// 3/4/5, the top 6; a letter that names none falls to 0, the bark, as the
/// original's "Invalid Tree Node Name" path stores. A later node of the
/// same slot replaces an earlier. **Names and letters compare without
/// case**: so read, 12 of 12 cached cells agree with the packet's
/// per-section vertex and index counts, and compared with case, none do
/// (`client::world::tests::the_leaves_are_the_packets_own_sections`).
pub fn slots(model: &crate::bh3::Model) -> [Option<Slot>; 6] {
    let mut out = [None; 6];
    for n in model.nodes.iter().filter(|n| n.num > 0) {
        let name = n.name.to_ascii_lowercase();
        let stem = name.get(..name.len().saturating_sub(2)).unwrap_or("");
        let last = name.chars().last().unwrap_or(' ');
        let pick = |letters: &[(char, usize)]| {
            letters
                .iter()
                .find(|(c, _)| *c == last)
                .map_or(0, |(_, t)| *t)
        };
        let (slot, texture) = match (name.as_str(), stem) {
            ("root", _) => (0, 0),
            ("trunk", _) => (1, 0),
            (_, "bottom_1") => (2, pick(&[('a', 1), ('b', 2)])),
            (_, "middle_1") => (3, pick(&[('a', 3), ('b', 4), ('c', 5)])),
            (_, "middle_2") => (4, pick(&[('a', 3), ('b', 4), ('c', 5)])),
            (_, "top_1") => (5, 6),
            _ => continue,
        };
        out[slot] = Some(Slot {
            start: n.start,
            num: n.num,
            texture,
        });
    }
    out
}

/// The texture each of `model`'s faces samples: the slot ([`slots`]) whose
/// run holds the face's first vertex; a face in no slot's run, the bark.
pub fn face_textures(model: &crate::bh3::Model) -> Vec<usize> {
    let slots = slots(model);
    model
        .faces
        .iter()
        .map(|f| {
            let v = f[0] as u32;
            slots
                .iter()
                .flatten()
                .find(|s| (s.start..s.start + s.num).contains(&v))
                .map_or(0, |s| s.texture)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// run1559b's packet, cell (6, 37), mask 6, all five slots, as
    /// `tools/render/tree_points.py` reads them off `place_tree` under
    /// unicorn: `(slot, x, y, model)`.
    const PACKET_6_37: [(i32, i32, i32, usize); 5] = [
        (0, 4750, 28719, 13),
        (1, 4857, 28947, 12),
        (2, 5038, 28785, 7),
        (3, 5221, 28589, 18),
        (4, 5221, 28947, 16),
    ];

    #[test]
    fn the_woodcutter_s_trees_stand_where_the_packet_stands_them() {
        for (slot, x, y, m) in PACKET_6_37 {
            assert_eq!(place(6, 37, 6, slot), (x, y), "slot {slot}");
            assert_eq!(model(x, y, 24), m, "slot {slot}'s model");
        }
    }

    /// Made to fail: slot 0 with the `break` the listing does not have
    /// lands elsewhere.
    #[test]
    fn slot_zero_falls_into_slot_one() {
        let (cx, cy, mask) = (6, 37, 6u8);
        let (mut x, mut y) = (cx * CELL + CELL / 2, cy * CELL + CELL / 2);
        let mut r = Random((x * 7 + y) as u32);
        x -= 96;
        y -= 144;
        x += r.get(-96, 0);
        y += r.get(-144, 0);
        assert_ne!((x, y), place(cx, cy, mask, 0));
    }

    /// `tools/render/tree_points.py`'s output on run1559b's packet, kept
    /// outside the repository.
    pub(crate) fn oracle() -> Option<String> {
        let path = std::env::var_os("RON_TREE_ORACLE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                    .join("ron-data/lab-experiments/2026-11-10-item-4080/run1559b-trees.txt")
            });
        std::fs::read_to_string(&path).ok()
    }

    /// **Every place the oracle printed, ours equal**: the sweep's 16 masks
    /// × 5 slots at four cells, and the 60 trees of the visible forest
    /// cells with their masks, read off the packet's own `forest` rows.
    #[test]
    fn every_place_the_packet_prints_is_ours() {
        let Some(text) = oracle() else {
            eprintln!("skipping: no tree oracle on disk");
            return;
        };
        let forest: std::collections::BTreeSet<(i32, i32)> = text
            .lines()
            .filter_map(|l| {
                let v: Vec<i32> = l
                    .strip_prefix("forest ")?
                    .split(' ')
                    .map(|t| t.parse().unwrap())
                    .collect();
                Some((v[0], v[1]))
            })
            .collect();
        let (mut sweeps, mut trees) = (0, 0);
        for l in text.lines() {
            let w: Vec<&str> = l.split(' ').collect();
            let n = |i: usize| w[i].parse::<i32>().unwrap();
            match w[0] {
                "sweep" => {
                    assert_eq!(place(n(1), n(2), n(3) as u8, n(4)), (n(5), n(6)), "{l}");
                    sweeps += 1;
                }
                "tree" => {
                    let mask = tree_mask(|x, y| forest.contains(&(x, y)), n(1), n(2));
                    assert_eq!(mask as i32, n(3), "{l}");
                    assert_eq!(place(n(1), n(2), mask, n(4)), (n(5), n(6)), "{l}");
                    assert_eq!(model(n(5), n(6), 24) as i32, n(8), "{l}");
                    trees += 1;
                }
                _ => {}
            }
        }
        assert_eq!((sweeps, trees, forest.len()), (320, 60, 189));
    }

    #[test]
    fn the_mask_reads_the_four_neighbours() {
        let forest = |x: i32, y: i32| (x, y) == (5, 37) || (x, y) == (6, 38);
        assert_eq!(tree_mask(forest, 6, 37), 1 | 8);
        assert_eq!(tree_mask(|_, _| false, 6, 37), 0);
    }
}
