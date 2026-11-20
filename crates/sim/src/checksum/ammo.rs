//! The occupied ammo pool in slot order. Each ordinary sprite contributes
//! its flags, its 99-byte display record and the path byte, and a round
//! that carries a nuke-arm spline its `SplineData` after them (item 4468).
use super::{Line, Range};
use crate::Sim;
use crate::combat::{Spline, SplineArm};
use crate::num::single::Single;

/// `Array<T>::increase_size` with increment −1, as 4465 read
/// `SimpleArray<int>`'s (`docs/CHECKSUM.md` §18): 4 from empty, then
/// doubling. run1721's nuke spline holds 12 control points in 16 and 121
/// points in 128.
fn grown(n: usize) -> usize {
    let mut cap = 0;
    while cap < n {
        cap = if cap == 0 { 4 } else { cap * 2 };
    }
    cap
}

/// **`SplineData::walk_data@009132b0`** of a nuke-arm spline (item 4468,
/// `docs/CHECKSUM.md` §18): `type`..`depth` (`+0x40`..`+0x64`, 36 bytes),
/// then `control_verts`, `knots`, `spline_knots`, `weights`,
/// `spline_verts` and `spline_normals`, each array as its `walk_data`
/// folds it — the length, and for a non-empty one the size, the
/// increment, the flags and the elements (`Array<Vector<float>>` one range
/// an element, `SimpleArray<float>` one for all). `None` for an arm whose
/// walk is not read.
fn spline_ranges(path: &Spline) -> Option<Vec<(&'static str, Vec<u8>)>> {
    let SplineArm::Nuke { s, e } = path.arm() else {
        return None;
    };
    let parts = Spline::nuke_parts(s, e);
    let n = parts.ctl.len();
    let (normals, length) = path.normals([parts.ctl[n - 2], parts.ctl[n - 1]]);
    let mut out = Vec::new();
    let mut head = Vec::with_capacity(36);
    for word in [
        3,                      // type: `generate_bspline`
        0x10,                   // flags: `Ammo::init`'s
        0,                      // max_control_depth_ratio
        length.bits() as i32,   // total_spline_length
        0,                      // last_knot
        0xbf80_0000_u32 as i32, // curr_dist, −1.0 (`Spline::clear`)
        0xbf80_0000_u32 as i32, // next_search_dist, −1.0
        -1,                     // search_scan
    ] {
        head.extend(word.to_le_bytes());
    }
    head.extend(3u16.to_le_bytes()); // degree
    head.extend((crate::air::MISSILE_FLIGHT as u16).to_le_bytes()); // depth
    out.push(("spline.type..depth", head));
    let header = |out: &mut Vec<(&'static str, Vec<u8>)>,
                  name: [&'static str; 4],
                  len: usize,
                  cap: usize| {
        out.push((name[0], (len as i32).to_le_bytes().to_vec()));
        if len > 0 {
            out.push((name[1], (cap as i32).to_le_bytes().to_vec()));
            out.push((name[2], (-1i16).to_le_bytes().to_vec()));
            out.push((name[3], vec![0]));
        }
    };
    let verts =
        |out: &mut Vec<(&'static str, Vec<u8>)>, name: [&'static str; 5], vs: &[[Single; 3]]| {
            header(
                out,
                [name[0], name[1], name[2], name[3]],
                vs.len(),
                grown(vs.len()),
            );
            for v in vs {
                out.push((
                    name[4],
                    v.iter().flat_map(|c| c.bits().to_le_bytes()).collect(),
                ));
            }
        };
    let floats =
        |out: &mut Vec<(&'static str, Vec<u8>)>, name: [&'static str; 5], fs: &[Single]| {
            header(
                out,
                [name[0], name[1], name[2], name[3]],
                fs.len(),
                fs.len(),
            );
            if !fs.is_empty() {
                out.push((
                    name[4],
                    fs.iter().flat_map(|c| c.bits().to_le_bytes()).collect(),
                ));
            }
        };
    verts(
        &mut out,
        [
            "control_verts.length",
            "control_verts.size",
            "control_verts.increment",
            "control_verts.flags",
            "control_verts[k]",
        ],
        &parts.ctl,
    );
    floats(
        &mut out,
        [
            "knots.length",
            "knots.size",
            "knots.increment",
            "knots.flags",
            "knots[..]",
        ],
        &parts.weights,
    );
    floats(
        &mut out,
        [
            "spline_knots.length",
            "spline_knots.size",
            "spline_knots.increment",
            "spline_knots.flags",
            "spline_knots[..]",
        ],
        &parts.knots,
    );
    floats(
        &mut out,
        [
            "weights.length",
            "weights.size",
            "weights.increment",
            "weights.flags",
            "weights[..]",
        ],
        &[],
    );
    verts(
        &mut out,
        [
            "spline_verts.length",
            "spline_verts.size",
            "spline_verts.increment",
            "spline_verts.flags",
            "spline_verts[k]",
        ],
        path.points(),
    );
    verts(
        &mut out,
        [
            "spline_normals.length",
            "spline_normals.size",
            "spline_normals.increment",
            "spline_normals.flags",
            "spline_normals[k]",
        ],
        &normals,
    );
    Some(out)
}

impl Sim {
    pub fn checksum_ammo(&self) -> Option<Line> {
        let mut ranges = Vec::with_capacity(self.combat.projectiles.len() * 3);
        for p in &self.combat.projectiles {
            // Guided paths and falling aircraft have separate display records.
            if p.graphic.guided || p.crash.is_some() {
                return None;
            }
            // A spline is walked after the path byte; an arm whose walk is
            // not read refuses the line.
            let path = match &p.spline {
                Some(path) => Some(spline_ranges(path)?),
                None => None,
            };
            let (o, _) = self.obj_o_who(p.shooter);
            // A gone target's pair stays on the round (item 4404).
            let (ox, whom) = p
                .target
                .map(|t| self.obj_o_who(t))
                .or(p.left)
                .unwrap_or((-1, -1));
            let flags = if p.spent {
                1
            } else {
                2 | (u8::from(p.rolling) << 2)
                    | (u8::from(p.missed) << 3)
                    | (u8::from(p.harmless) << 4)
            };
            let mut body = Vec::with_capacity(99);
            body.push(0); // rolling display byte (separate from flag 4)
            body.extend((p.accuracy as i16).to_le_bytes());
            for word in [
                p.gpiece,
                p.launch.x,
                p.launch.y,
                p.sz,
                p.landing.x,
                p.landing.y,
                p.ez,
                p.cur_time,
                p.total_time,
                p.angle.0,
                p.splash_area,
                p.slot as i32,
                p.graphic.graph_index as i32,
                i32::from(p.owner),
                o as i32,
                p.num_guys,
                whom as i32,
                ox as i32,
                // traj: `(local_30 != 0) + 1` (`Ammo::init:831`), 2 for a
                // round on a spline (item 4468).
                1 + i32::from(path.is_some()),
                p.v1z.bits() as i32,
                p.graphic.dx.bits() as i32,
                0,
                0,
                0, // bank_dx, bank_dy, start_roll_angle
            ] {
                body.extend(word.to_le_bytes());
            }
            for (field, bytes) in [
                ("flags", vec![flags]),
                ("rolling..start_roll_angle", body),
                ("has_path", vec![u8::from(path.is_some())]),
            ]
            .into_iter()
            .chain(path.into_iter().flatten())
            {
                ranges.push(Range {
                    record: p.slot as i16,
                    field,
                    bytes,
                });
            }
        }
        Some(Line {
            name: "ammo",
            ranges,
        })
    }
}
