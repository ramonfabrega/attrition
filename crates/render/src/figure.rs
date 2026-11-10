//! **A figure's mesh, posed at rest and placed** (item 4036): the decoded
//! `.BH3`'s vertices carried out of their nodes into the model's space, then
//! turned to the figure's facing and moved to its draw-list anchor.
//!
//! **The rest pose.** A vertex is stored in its node's own space (the
//! citizen's spine runs along its node's x from 0 to 9). Each node's
//! transform is `Transform<float>::fill_matrix` of its loaded rotation and
//! translation (`Node::loaded_rotation`, `loaded_translation`), and a node
//! sits in its parent's: `vertex · node · parent · … · root`. No animation
//! is applied — `.BHa` keys are §6's "After these", and an animation's tree
//! need not match its model's (`docs/FORMATS.md` §3).
//!
//! **The placement** is read from `GuyOut::draw@005dc490`: the angle drawn
//! is the figure's facing in degrees plus 180 (`__real_43340000`), turned
//! with `Transform<float>::rotate_z`, and the piece's scale multiplies the
//! model. The scale is `RData +0x88` (the `<UNIT>`'s `scale=`) times the
//! global `guy_scale`, read off run1559b's packet (item 4065); the chad's
//! `sy = −1` is not applied. What `GraphicPieces::draw` does past that —
//! the bank, the pivots — is not read, and no bank is drawn
//! (`docs/RENDERER.md` §3.6).

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
            normals: vec![[0.0; 4]; 2],
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
