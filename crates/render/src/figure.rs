//! **A figure's mesh, posed at rest and placed** (item 4036): the decoded
//! `.BH3`'s vertices carried out of their nodes into the model's space, then
//! turned to the figure's facing and moved to its draw-list anchor.
//!
//! **The rest pose.** A vertex is stored in its node's own space (the
//! citizen's spine runs along its node's x from 0 to 9). Each node's
//! transform is `Transform<float>::fill_matrix` of its loaded rotation and
//! translation (`Node::loaded_rotation`, `loaded_translation`), and a node
//! sits in its parent's: `vertex · node · parent · … · root`.
//!
//! **The pose** ([`posed`], item 4067): an animation's key is each node's
//! `local` transform *inside* that rest transform, the trees paired by
//! place (`HierObj::set_to_time@00536e30`, `update_transforms@00536c60`).
//!
//! **The placement** is read from `GuyOut::draw@005dc490`: the angle drawn
//! is `fast_angle_to_degrees` of `GuyData::angle` plus 180
//! (`__real_43340000`), turned with `Transform<float>::rotate_z`, and the
//! piece's scale multiplies the model. The scale is `RData +0x88` (the
//! `<UNIT>`'s `scale=`) times the global `guy_scale`, read off run1559b's
//! packet (item 4065). A figure's turn and its mirror are the original's
//! model-to-world map, `get_position`'s, as [`guy_placement`] draws it
//! (item 4069); a building keeps [`placement`]. What `GraphicPieces::draw`
//! does past that — the bank, the pivots — is not read, and no bank is
//! drawn (`docs/RENDERER.md` §3.6).

use crate::bh3::Model;
use crate::camera::{Mat4, fill, rotate};

/// The model's vertices in its own space, at rest. A node whose run lies
/// past the vertices (the decoder refuses such a file) contributes none.
pub fn rest_pose(model: &Model) -> Vec<[f32; 3]> {
    let mut world: Vec<Mat4> = Vec::with_capacity(model.nodes.len());
    let mut out = model.verts.clone();
    for n in &model.nodes {
        let local = fill(Some(n.loaded_rotation()), n.loaded_translation());
        let m = match n.parent {
            Some(p) => Mat4::then(&local, &world[p]),
            None => local,
        };
        let (s, e) = (n.start as usize, (n.start + n.num) as usize);
        if let Some(run) = model.verts.get(s..e) {
            for (o, v) in out[s..e].iter_mut().zip(run) {
                *o = apply(&m, *v);
            }
        }
        world.push(m);
    }
    out
}

/// **The model posed at `ms` into an animation** (item 4067), read from
/// `HierObj::set_to_time@00536e30` and `HierObj::update_transforms
/// @00536c60`.
///
/// - **The key sits inside the rest transform, it does not replace it.** A
///   node's world transform is `parent.world · parent2local · local`:
///   `parent2local` is the `.BH3`'s loaded rotation and translation (the
///   rest pose, [`rest_pose`]), and `local` is what `set_to_time` writes
///   from the keys. In this module's row-vector order a vertex goes
///   `v · key · rest · parent`. A citizen's calf key carries no translation
///   because its bone length is the rest's; the pelvis key's
///   `(−22.3, −3.2, −0.48)` is `−restᵀ · t_rest`, which seats the pelvis
///   on `Bip01` as the animation lifts `Bip01` to the hip.
/// - **The pairing is by place in the tree, not by name.** `set_to_time`
///   walks the model's hierarchy and the animation's in step — a node's
///   child with the animation node's child, its sibling with its sibling —
///   so the model's first root takes the played root, its `k`-th child the
///   animation node's `k`-th child. A model node with no counterpart has
///   its `local` reset to the identity (`Transform<float>::~Transform`), and
///   so have all of its descendants: they stand at rest.
/// - **The keys.** `ms` is first reduced modulo the node's own last key time
///   (`AnimObj::anim_time`, the accumulated key durations). The two keys
///   bracketing it are found on the cumulative times
///   ([`crate::bha::AnimNode::times`]), and the transform between them is
///   the rotation's component-wise blend, the second quaternion negated
///   when the two point into opposite halves (`Transform<float>::set
///   @00421dc0`, no normalisation), and the translation's linear blend.
///   `load_hier`'s key flags (a key whose rotation or translation does not
///   move to the next, within 0.01 and 0.1) only skip a blend that would
///   change nothing at that tolerance, and are not reproduced.
pub fn posed(model: &Model, anim: &crate::bha::Anim, ms: u32) -> Vec<[f32; 3]> {
    let world = worlds(model, anim, ms);
    let mut out = model.verts.clone();
    for (n, m) in model.nodes.iter().zip(&world) {
        let (s, e) = (n.start as usize, (n.start + n.num) as usize);
        if let Some(run) = model.verts.get(s..e) {
            for (o, v) in out[s..e].iter_mut().zip(run) {
                *o = apply(m, *v);
            }
        }
    }
    out
}

/// Each node's world transform at `ms`, `parent.world · parent2local ·
/// local` ([`posed`]).
fn worlds(model: &Model, anim: &crate::bha::Anim, ms: u32) -> Vec<Mat4> {
    let keys = keyed(model, anim, ms);
    let mut world: Vec<Mat4> = Vec::with_capacity(model.nodes.len());
    for (i, n) in model.nodes.iter().enumerate() {
        let rest = fill(Some(n.loaded_rotation()), n.loaded_translation());
        let local = match keys[i] {
            Some(k) => Mat4::then(&fill(Some(k.rotation), k.translation), &rest),
            None => rest,
        };
        let m = match n.parent {
            Some(p) => Mat4::then(&local, &world[p]),
            None => local,
        };
        world.push(m);
    }
    world
}

/// **Each node's origin in the model's space at `ms`** (item 4092): the
/// translation of its world transform, `HierObj::world_transform.trans`
/// (+0x74) after `update_all_transforms@00536db0`.
pub fn joints(model: &Model, anim: &crate::bha::Anim, ms: u32) -> Vec<[f32; 3]> {
    worlds(model, anim, ms)
        .iter()
        .map(|m| apply(m, [0.0; 3]))
        .collect()
}

/// One model node's `local` transform at a time, as [`posed`] takes it
/// (item 4092): the two keys blended — `HierObj::iKey1`/`iKey2` (+0xac,
/// +0xae), which `set_to_time@00536e30` leaves on the node — and the
/// blend's rotation and translation (`local_transform`, +0x34).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyed {
    pub keys: (usize, usize),
    pub rotation: [f32; 4],
    pub translation: [f32; 3],
}

/// **Each model node's key at `ms`** (item 4092), in the model's node
/// order: `None` for a node the animation's tree has no place for (it
/// stands at rest, [`posed`]). A one-key node reports `(0, 0)`.
pub fn keyed(model: &Model, anim: &crate::bha::Anim, ms: u32) -> Vec<Option<Keyed>> {
    let played = anim.played();
    let mut partner: Vec<Option<usize>> = vec![None; model.nodes.len()];
    let mut taken: Vec<usize> = vec![0; played.len()];
    let mut first_root = true;
    for (i, n) in model.nodes.iter().enumerate() {
        partner[i] = match n.parent {
            None => {
                let r = first_root && !played.is_empty();
                first_root = false;
                r.then_some(0)
            }
            Some(p) => partner[p].and_then(|a| {
                let k = taken[a];
                taken[a] += 1;
                played
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| c.parent == Some(a))
                    .nth(k)
                    .map(|(j, _)| j)
            }),
        };
    }
    partner
        .iter()
        .map(|p| {
            p.map(|a| {
                let (rotation, translation, keys) = sample_keyed(&played[a], ms);
                Keyed {
                    keys,
                    rotation,
                    translation,
                }
            })
        })
        .collect()
}

/// One animation node's rotation and translation at `ms` ([`posed`]).
pub fn sample(node: &crate::bha::AnimNode, ms: u32) -> ([f32; 4], [f32; 3]) {
    let (q, t, _) = sample_keyed(node, ms);
    (q, t)
}

/// [`sample`] with the two keys it blends, `iKey1` and `iKey2`.
fn sample_keyed(node: &crate::bha::AnimNode, ms: u32) -> ([f32; 4], [f32; 3], (usize, usize)) {
    let keys = &node.keys;
    let rest = || ([0.0, 0.0, 0.0, 1.0], [0.0; 3], (0, 0));
    let Some(first) = keys.first() else {
        return rest();
    };
    if keys.len() == 1 {
        return (first.loaded_rotation(), first.loaded_translation(), (0, 0));
    }
    let times = node.times();
    let n = keys.len();
    let span = u32::from(*times.last().unwrap_or(&0));
    let t = if span == 0 { 0 } else { ms % span };
    let at = |k: usize| u32::from(times[k]);
    // The bracketing pair: the last key time below `t`, and the next.
    let (mut lo, mut hi) = (0usize, n);
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if at(mid) < t { lo = mid } else { hi = mid }
    }
    if hi == n {
        (lo, hi) = (n - 2, n - 1);
    }
    let (a, b, k1, k2) = if at(lo) < t {
        (at(lo), at(hi), lo + 1, hi + 1)
    } else {
        let a = if lo == 0 { 0 } else { at(lo - 1) };
        (a, at(hi - 1), lo, hi)
    };
    let k1 = k1.min(n - 1);
    let k2 = if k2 >= n { 0 } else { k2 };
    let f = if b > a {
        (t.saturating_sub(a)) as f32 / (b - a) as f32
    } else {
        0.0
    };
    let (q1, mut q2) = (keys[k1].loaded_rotation(), keys[k2].loaded_rotation());
    if q1.iter().zip(&q2).map(|(x, y)| x * y).sum::<f32>() < 0.0 {
        q2 = q2.map(|v| -v);
    }
    let g = 1.0 - f;
    let (t1, t2) = (keys[k1].loaded_translation(), keys[k2].loaded_translation());
    (
        [0, 1, 2, 3].map(|i| q1[i] * g + q2[i] * f),
        [0, 1, 2].map(|i| t1[i] * g + t2[i] * f),
        (k1, k2),
    )
}

/// `p · m` for a point.
pub fn apply(m: &Mat4, p: [f32; 3]) -> [f32; 3] {
    let m = &m.0;
    [
        p[0] * m[0] + p[1] * m[4] + p[2] * m[8] + m[12],
        p[0] * m[1] + p[1] * m[5] + p[2] * m[9] + m[13],
        p[0] * m[2] + p[1] * m[6] + p[2] * m[10] + m[14],
    ]
}

/// A facing, a full turn in `2^32` (`sim::Angle`), in whole degrees as
/// `fast_angle_to_degrees` gives them to `rotate_z`.
pub fn facing_degrees(facing: u32) -> i32 {
    ((facing as u64 * 360) >> 32) as i32
}

/// The model-to-world transform of a figure: scaled, turned to
/// `facing + 180°` about z, and moved to `at`.
pub fn placement(at: [i32; 3], facing: u32, scale: f32) -> Mat4 {
    let turn = fill(
        rotate(None, facing_degrees(facing) + 180, [0.0, 0.0, 1.0]),
        [at[0] as f32, at[1] as f32, at[2] as f32],
    );
    let mut s = Mat4::IDENTITY;
    s.0[0] = scale;
    s.0[5] = scale;
    s.0[10] = scale;
    Mat4::then(&s, &turn)
}

/// **A figure's model-to-world transform** (item 4069): the model's y
/// negated, then turned by `drawn` whole degrees, with x towards y, scaled
/// and moved to `at`. A model point `m` lands at
/// `at + s·(m.x·c + m.y·s_, m.x·s_ − m.y·c, m.z)`, `c, s_` the cosine and
/// sine of `drawn`.
///
/// `drawn` is the angle `GuyOut::draw@005dc490` hands `rotate_z`:
/// `fast_angle_to_degrees` of `GuyData::angle` (`+0x18`) blended from
/// `last_angle` (`+0x1c`) by `graphic_percent` (listing `5dcac4`–`5dcb3b`),
/// plus 180 (`5dcb45`, `__real_43340000`), into `GraphicChad::angle`
/// (`+0x3c`), which `draw_unit@008e7da0` truncates into `rotate_z`
/// (`8e8887`). The client draws at a blend of 0 (a sim frame's own angle),
/// which is [`sim::data::pivot::release_rotation`]'s degree.
///
/// The sense and the mirror are the original's own model-to-world map,
/// `GraphicPieces::get_position@0090b750`, as `sim::data::pivot::
/// release_offset_at` ports it, packet-backed at all 256 headings
/// (`v = s·R(d)·P` with y negated, `d` that same `fast_angle_to_degrees +
/// 180`). [`placement`]'s quaternion turn, applied with no mirror, is that
/// map reflected in world y through the anchor: a sign flip of every
/// figure's heading (parked 4067a's quarter turn on figures facing
/// near −45°).
pub fn guy_placement(at: [i32; 3], drawn: i32, scale: f32) -> Mat4 {
    let r = (drawn.rem_euclid(360) as f32).to_radians();
    let (c, s) = (r.cos() * scale, r.sin() * scale);
    let mut m = Mat4::IDENTITY;
    m.0[0] = c;
    m.0[1] = s;
    m.0[4] = s;
    m.0[5] = -c;
    m.0[10] = scale;
    m.0[12] = at[0] as f32;
    m.0[13] = at[1] as f32;
    m.0[14] = at[2] as f32;
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bh3::Node;

    fn node(name: &str, parent: Option<usize>, start: u32, num: u32, t: [f32; 3]) -> Node {
        Node {
            name: name.into(),
            parent,
            start,
            num,
            rotation: [0.0, 0.0, 0.0, 1.0],
            translation: t,
            trans_w: 0.0,
        }
    }

    #[test]
    fn a_child_s_vertices_sit_in_its_parent_s_space() {
        let mut child = node("child", Some(0), 1, 1, [0.0, 10.0, 0.0]);
        // A quarter turn about z: the child's x runs along the model's y.
        let h = std::f32::consts::FRAC_1_SQRT_2;
        child.rotation = [0.0, 0.0, h, h];
        let model = Model {
            verts: vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            normals: vec![[0.0; 3]; 2],
            texcoords: vec![[0.0; 2]; 2],
            faces: vec![],
            nodes: vec![node("root", None, 0, 1, [0.0, 0.0, 5.0]), child],
        };
        let rest = rest_pose(&model);
        let close = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01);
        assert!(close(rest[0], [1.0, 0.0, 5.0]), "{:?}", rest[0]);
        assert!(
            close(rest[1], [0.0, 12.0, 5.0]) || close(rest[1], [0.0, 8.0, 5.0]),
            "{:?}",
            rest[1]
        );
    }

    fn key(seconds: f32, rotation: [f32; 4], translation: [f32; 3]) -> crate::bha::Key {
        crate::bha::Key {
            seconds,
            rotation,
            translation,
            unread: 0,
        }
    }

    fn track(parent: Option<usize>, keys: Vec<crate::bha::Key>) -> crate::bha::AnimNode {
        crate::bha::AnimNode {
            parent,
            keys,
            spare: 0,
        }
    }

    fn close(a: [f32; 3], b: [f32; 3]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.02)
    }

    /// A root, and a child turned a half turn about (−1, 0, −1)/√2 and
    /// carried 22.3 up — the citizen's pelvis at rest.
    fn pelvis_model() -> Model {
        let mut child = node("pelvis", Some(0), 1, 1, [0.0, 0.0, 22.3]);
        child.rotation = [-0.707, 0.0, -0.707, 0.0];
        Model {
            verts: vec![[0.0; 3], [0.0; 3]],
            normals: vec![[0.0; 3]; 2],
            texcoords: vec![[0.0; 2]; 2],
            faces: vec![],
            nodes: vec![node("root", None, 0, 1, [0.0; 3]), child],
        }
    }

    #[test]
    fn an_identity_key_leaves_the_rest_pose() {
        let model = pelvis_model();
        let i = key(0.0, [0.0, 0.0, 0.0, 1.0], [0.0; 3]);
        let anim = crate::bha::Anim {
            nodes: vec![track(None, vec![i]), track(Some(0), vec![i])],
            roots: 1,
            tracks: vec![],
        };
        let (rest, posed) = (rest_pose(&model), posed(&model, &anim, 500));
        assert!(
            rest.iter().zip(&posed).all(|(a, b)| close(*a, *b)),
            "{posed:?}"
        );
    }

    /// **The key sits inside the rest transform** (`HierObj::update_transforms
    /// @00536c60`: `parent.world · parent2local · local`). The pelvis key
    /// `−restᵀ · t_rest` seats the pelvis on its parent's origin; applied
    /// outside the rest instead, it would land the pelvis 22 units away.
    #[test]
    fn the_key_is_applied_inside_the_rest_transform() {
        let model = pelvis_model();
        let k = key(0.0, [0.0, 0.0, 0.0, 1.0], [-22.3, 0.0, 0.0]);
        let anim = crate::bha::Anim {
            nodes: vec![
                track(None, vec![key(0.0, [0.0, 0.0, 0.0, 1.0], [0.0; 3])]),
                track(Some(0), vec![k]),
            ],
            roots: 1,
            tracks: vec![],
        };
        assert!(close(rest_pose(&model)[1], [0.0, 0.0, 22.3]));
        let p = posed(&model, &anim, 0);
        assert!(close(p[1], [0.0, 0.0, 0.0]), "{:?}", p[1]);
    }

    #[test]
    fn a_node_the_animation_lacks_stands_at_rest() {
        let model = pelvis_model();
        let anim = crate::bha::Anim {
            nodes: vec![track(
                None,
                vec![key(0.0, [0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 5.0])],
            )],
            roots: 1,
            tracks: vec![],
        };
        let p = posed(&model, &anim, 0);
        // The root is lifted 5; the pelvis keeps its rest place under it.
        assert!(close(p[0], [0.0, 0.0, 5.0]), "{:?}", p[0]);
        assert!(close(p[1], [0.0, 0.0, 27.3]), "{:?}", p[1]);
    }

    /// Between two keys the translation blends linearly, and a rotation in
    /// the far hemisphere is negated before the blend. **A key's `seconds`
    /// is the time after it**: `set_to_time` steps both indices on by one
    /// once the clock passes the lower time, so key `k` is reached at
    /// `times[k − 1]` and key 0 at zero, and past the last time the clip
    /// blends back to key 0.
    #[test]
    fn a_clock_between_two_keys_blends_them() {
        let n = track(
            None,
            vec![
                key(0.5, [0.0, 0.0, 0.0, 1.0], [0.0; 3]),
                key(0.5, [0.0, 0.0, 0.0, -1.0], [10.0, 0.0, 0.0]),
                key(0.0, [0.0, 0.0, 0.0, 1.0], [20.0, 0.0, 0.0]),
            ],
        );
        assert_eq!(n.times(), vec![500, 1000, 1000]);
        let (q, t) = sample(&n, 250);
        assert!(close(t, [5.0, 0.0, 0.0]), "{t:?}");
        assert!((q[3].abs() - 1.0).abs() < 1e-3, "{q:?}");
        let (q, t) = sample(&n, 750);
        assert!(close(t, [15.0, 0.0, 0.0]), "{t:?}");
        assert!((q[3].abs() - 1.0).abs() < 1e-3, "{q:?}");
    }

    #[test]
    fn a_facing_turns_half_a_turn_further_and_lands_on_the_anchor() {
        assert_eq!(facing_degrees(0), 0);
        assert_eq!(facing_degrees(1 << 30), 90);
        assert_eq!(facing_degrees(u32::MAX), 359);
        let m = placement([100, 200, 30], 0, 2.0);
        let p = apply(&m, [1.0, 0.0, 0.0]);
        assert!(
            (p[0] - 98.0).abs() < 1e-3 && (p[1] - 200.0).abs() < 1e-3 && p[2] == 30.0,
            "{p:?}"
        );
    }
}
