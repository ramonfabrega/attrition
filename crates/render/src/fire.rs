//! **A damaged building's fire and smoke: where the original anchors them**
//! (item 4201; `docs/RENDERER.md` §3.20).
//!
//! Read from `Wall::inc_time@0063fb60` (a building's `inc_time`: `Build`'s
//! vtable `+0xa0`), `ObjectData::health_level@006534b0` (`+0x118`),
//! `BuildData::hits@0062e740` (`+0x11c`), `GraphicPieces::emit_build_particles
//! @00909590`, `GraphicPieces::get_position@0090b750`,
//! `GraphicPieces::init_position@00901820`, `GraphicEvents::verify_load
//! @008e4780` and `BH3Method::load_hier@00915010`:
//!
//! - **The ladder.** On every finished building, each frame, the health
//!   level decides which of the model's attach slots burn: from level 2
//!   (under 75 % of its hits) slot 11, from 3 slot 10, from 4 slot 9, from 5
//!   slot 8 ([`burning_slots`]). Each burning slot is two emitters at one
//!   point, `quick_particle_lookups[0x86]` and `[0x85]`: `effects_graphics.xml`'s
//!   `BUILDING FIRE` and `BUILDING SMOKE` (internal strings 3789 and 3788).
//! - **The slots.** `load_hier` files a node named `ammo node i` or `node i`
//!   in slot `i`, `pivot node i` in `i + 4` and `smoke node i` in `i + 8`
//!   ([`attach_slots`]): the burning slots are the four smoke nodes. A
//!   building's are in its `<BUILD>`'s `zbuffer=` model.
//! - **The point.** `verify_load` records each slot's place at the rest
//!   pose, moved `32 · halfoffset` on x and y (`BuildRDataStruct::
//!   build_half_tcoord`, `RData +0x98 → +0x2c`). `get_position` turns it by
//!   270° about z, negates y and scales by `build_scale` (3.0); the emitter
//!   stands at the building's point plus that ([`anchor`]).
//!
//! Diffed on run1559b's and run1570's packets (`docs/RENDERER.md` §3.20): the
//! city `0/2000` burns on all four slots, and every emitter's `desc.pos` is
//! this arithmetic on `village_6tz.bh3`'s nodes. The particles themselves —
//! their timing, motion and blend — are not read.

use crate::bh3::Model;
use crate::draw::PlacedBuilding;
use crate::figure::apply;

/// `?build_scale@@3MA` (`0x00c062d0`), read off run1559b's packet: 3.0.
pub const BUILD_SCALE: f32 = 3.0;

/// The attach slots `NodePacket` keeps (`HierObj *nodes[14]`).
pub const SLOTS: usize = 14;

/// The world units `init_position` moves a building's slot by on x and on
/// y, per `halfoffset`: half a tile at the model's scale.
pub const HALF_SHIFT: f32 = 32.0;

/// **`ObjectData::health_level`**: 0 at 90 % of `max` or more, 1 at 75 %,
/// 2 at 50 %, 3 at 25 %, 4 at 10 %, 5 below that with a hit left, 6 at
/// none. `left` is `max − damage`, clamped to `0..=max`; a negative `max`
/// leaves none. The thresholds are C's integer `max · k / 100`.
pub fn health_level(max: i32, damage: i32) -> u8 {
    let mut left = max - damage;
    if left < 0 || max < 0 {
        left = 0;
    } else if max < left {
        left = max;
    }
    let at = |k: i32| max * k / 100;
    if at(90) <= left {
        0
    } else if at(75) <= left {
        1
    } else if at(50) <= left {
        2
    } else if at(25) <= left {
        3
    } else if at(10) <= left {
        4
    } else if left < 1 {
        6
    } else {
        5
    }
}

/// The slots `Wall::inc_time` burns at a health level, in its order.
pub fn burning_slots(level: u8) -> &'static [u8] {
    const LADDER: [u8; 4] = [11, 10, 9, 8];
    let n = match level {
        0 | 1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 4,
    };
    &LADDER[..n]
}

/// **Each attach slot's node, at the rest pose**: its origin in the model's
/// space, by `load_hier`'s names (`int_str_array` strings 220–223, `ammo
/// node `, `pivot node `, `smoke node `, `node `, each followed by its
/// number and compared whole); a slot no node names is `None`. A later node
/// that names a slot again takes it, as `NodePacket::set_node` overwrites.
pub fn attach_slots(model: &Model) -> [Option<[f32; 3]>; SLOTS] {
    let origins = rest_origins(model);
    let mut out = [None; SLOTS];
    for (n, o) in model.nodes.iter().zip(origins) {
        if let Some(slot) = slot_of(&n.name)
            && slot < SLOTS
        {
            out[slot] = Some(o);
        }
    }
    out
}

/// The slot `load_hier` files a node name in: for each `i` below 14 in
/// turn, `ammo node i` → `i`, `pivot node i` → `i + 4`, `smoke node i` →
/// `i + 8`, `node i` → `i`; the first that matches.
fn slot_of(name: &str) -> Option<usize> {
    for i in 0..SLOTS {
        if name == format!("ammo node {i}") {
            return Some(i);
        }
        if name == format!("pivot node {i}") {
            return Some(i + 4);
        }
        if name == format!("smoke node {i}") {
            return Some(i + 8);
        }
        if name == format!("node {i}") {
            return Some(i);
        }
    }
    None
}

/// Every node's origin in the model's space at rest, as
/// [`crate::figure::rest_pose`] carries a vertex.
fn rest_origins(model: &Model) -> Vec<[f32; 3]> {
    let mut world: Vec<crate::camera::Mat4> = Vec::with_capacity(model.nodes.len());
    for n in &model.nodes {
        let local = crate::camera::fill(Some(n.loaded_rotation()), n.loaded_translation());
        let m = match n.parent {
            Some(p) => crate::camera::Mat4::then(&local, &world[p]),
            None => local,
        };
        world.push(m);
    }
    world.iter().map(|m| apply(m, [0.0; 3])).collect()
}

/// **The emitter's point** for a slot at `origin` (its node at rest, or
/// `None` where the model names no node there: `init_position` then
/// records a place of zero, unshifted) on building `b`: `s = origin +
/// 32 · halfoffset` on x and y, then `b`'s point plus
/// `BUILD_SCALE · (−s.y, −s.x, s.z)`.
pub fn anchor(b: &PlacedBuilding, origin: Option<[f32; 3]>) -> [f32; 3] {
    let s = match origin {
        Some([x, y, z]) => {
            let h = HALF_SHIFT * b.half_offset as f32;
            [x + h, y + h, z]
        }
        None => [0.0; 3],
    };
    [
        b.x as f32 - BUILD_SCALE * s[1],
        b.y as f32 - BUILD_SCALE * s[0],
        b.z as f32 + BUILD_SCALE * s[2],
    ]
}

/// One burning slot: a building's fire and smoke emitters at one point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Emitter {
    pub who: u8,
    pub o: i16,
    /// The attach slot, 8 to 11.
    pub slot: u8,
    pub pos: [f32; 3],
}

/// **The building's burning slots** (`Wall::inc_time`): none before it is
/// finished; otherwise one [`Emitter`] a slot of its health level's
/// [`burning_slots`], from `slots` (its model's [`attach_slots`]). The
/// level reads `construct_hits` against `hits − health`; the queued
/// upgrade's arm of `BuildData::hits` is not taken (parked 4197c).
pub fn emitters(b: &PlacedBuilding, slots: &[Option<[f32; 3]>; SLOTS]) -> Vec<Emitter> {
    if !b.active {
        return Vec::new();
    }
    let level = health_level(b.construct_hits, b.hits - b.health);
    burning_slots(level)
        .iter()
        .map(|&slot| Emitter {
            who: b.who,
            o: b.o,
            slot,
            pos: anchor(b, slots[slot as usize]),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bh3::Node;

    /// The thresholds, each side of each, on run1559b's city's 1,200.
    #[test]
    fn the_health_levels_fall_where_object_data_s_do() {
        let at = |left: i32| health_level(1200, 1200 - left);
        let levels: Vec<u8> = [
            1200, 1080, 1079, 900, 899, 600, 599, 300, 299, 120, 119, 1, 0,
        ]
        .map(at)
        .to_vec();
        assert_eq!(levels, [0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6]);
        // run1559b's packet: 1,107 of 1,200 taken, 93 left.
        assert_eq!(health_level(1200, 1107), 5);
        // Damage past the hits leaves none; a heal past them is the hits.
        assert_eq!(health_level(1200, 5000), 6);
        assert_eq!(health_level(1200, -50), 0);
        assert_eq!(burning_slots(1), [] as [u8; 0]);
        assert_eq!(burning_slots(2), [11]);
        assert_eq!(burning_slots(5), [11, 10, 9, 8]);
        assert_eq!(burning_slots(6), [11, 10, 9, 8]);
    }

    fn node(name: &str, parent: Option<usize>, t: [f32; 3]) -> Node {
        Node {
            name: name.into(),
            parent,
            start: 0,
            num: 0,
            rotation: [0.0, 0.0, 0.0, 1.0],
            translation: t,
            trans_w: 0.0,
        }
    }

    /// `load_hier`'s names: `smoke node i` is slot `i + 8`, `pivot node i`
    /// slot `i + 4`, a child carried by its parent's place.
    #[test]
    fn the_slots_are_load_hier_s() {
        let model = Model {
            verts: Vec::new(),
            normals: Vec::new(),
            texcoords: Vec::new(),
            faces: Vec::new(),
            nodes: vec![
                node("root", None, [10.0, 0.0, 0.0]),
                node("smoke node 0", Some(0), [67.2, -45.09, 63.58]),
                node("pivot node 3", Some(0), [1.0, 2.0, 3.0]),
                node("smoke node 3", Some(0), [-61.76, -45.33, 48.89]),
                node("Smoke Node 1", Some(0), [0.0; 3]),
            ],
        };
        let s = attach_slots(&model);
        // `load_hier` keeps the translation to hundredths, truncated.
        let near = |p: Option<[f32; 3]>, q: [f32; 3]| {
            p.is_some_and(|p| (0..3).all(|k| (p[k] - q[k]).abs() < 0.02))
        };
        assert!(near(s[8], [77.2, -45.09, 63.58]), "{:?}", s[8]);
        assert!(near(s[7], [11.0, 2.0, 3.0]), "{:?}", s[7]);
        assert!(near(s[11], [10.0 - 61.76, -45.33, 48.89]), "{:?}", s[11]);
        // Compared whole: the case is the file's.
        assert_eq!(s[9], None);
    }

    /// run1559b's packet: `AttachPos` 37, slot 8 of piece 57144, is
    /// (99.2, −13.09, 63.58), and its emitter stands at (3207.27, 30518.4,
    /// 726.74) on the city at (3168, 30816, 536). Made to fail: unturned,
    /// it stands 400 units off.
    #[test]
    fn the_anchor_is_get_position_s() {
        let city = PlacedBuilding {
            who: 0,
            o: 2000,
            x: 3168,
            y: 30816,
            z: 536,
            type_index: 0,
            orig_type: 0,
            piece: 57144,
            half_offset: 1,
            active: true,
            job_counter: 0,
            construct_time: 0,
            field: None,
            started: true,
            foundation: None,
            scaffold: None,
            age: 0,
            health: 93,
            hits: 1200,
            construct_hits: 1200,
        };
        let a = anchor(&city, Some([67.2, -45.09, 63.58]));
        let near = |p: [f32; 3], q: [f32; 3]| (0..3).all(|k| (p[k] - q[k]).abs() < 0.05);
        assert!(near(a, [3207.27, 30518.4, 726.74]), "{a:?}");
        let mut slots = [None; SLOTS];
        slots[8] = Some([67.2, -45.09, 63.58]);
        let e = emitters(&city, &slots);
        assert_eq!(e.iter().map(|e| e.slot).collect::<Vec<_>>(), [11, 10, 9, 8]);
        // A slot no node names stands on the building's own point.
        assert_eq!(e[0].pos, [3168.0, 30816.0, 536.0]);
        assert!(near(e[3].pos, a));
        // Unfinished, it does not burn.
        assert!(
            emitters(
                &PlacedBuilding {
                    active: false,
                    ..city
                },
                &slots
            )
            .is_empty()
        );
        // Unturned: (x + 3·s.x, y + 3·s.y) parts by hundreds of units.
        let unturned = [3168.0 + 3.0 * 99.2, 30816.0 - 3.0 * 13.09];
        assert!((unturned[0] - a[0]).hypot(unturned[1] - a[1]) > 300.0);
    }
}
