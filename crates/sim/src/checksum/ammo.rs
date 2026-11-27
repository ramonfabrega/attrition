//! The occupied ammo pool in slot order. Each ordinary sprite contributes
//! its flags, its 99-byte display record and the path byte, and a round
//! that carries a nuke-arm (item 4468) or `calc_from_dir` (item 4495)
//! spline its `SplineData` after them.
use super::{Line, Range};
use crate::Sim;
use crate::combat::{Spline, SplineArm};
use crate::num::single::Single;

/// **The 24 words of an ammo sprite's 99-byte range**, in the order
/// [`Sim::checksum_ammo`] writes them after `rolling` (one byte) and the
/// accuracy (two): word `k` sits at byte `3 + 4k`. The names are this
/// crate's fields, so a reader of a parted range names its byte by this
/// table (`rondata::checksum::fields`, the pass of 2026-11-27: a probe and
/// eight tests decoded the range with a closure of their own, and the
/// scorer printed an ammo leaf as bare bytes). A test holds it to the writer.
pub const AMMO_WORDS: [&str; 24] = [
    "gpiece",
    "launch_x",
    "launch_y",
    "sz",
    "land_x",
    "land_y",
    "ez",
    "cur_time",
    "total_time",
    "angle",
    "splash_area",
    "slot",
    "graph_index",
    "owner",
    "o",
    "num_guys",
    "whom",
    "ox",
    "traj",
    "v1z",
    "dx",
    "bank_dx",
    "bank_dy",
    "start_roll_angle",
];

/// An arm's walk: `max_control_depth_ratio`, the degree, the depth, the
/// control points, `knots` and `spline_knots`.
type ArmWalk<'a> = (i32, u16, u16, &'a [[Single; 3]], &'a [Single], &'a [Single]);

/// **`SplineData::walk_data@009132b0`** of a nuke-arm spline (item 4468,
/// `docs/CHECKSUM.md` §18): `type`..`depth` (`+0x40`..`+0x64`, 36 bytes),
/// then `control_verts`, `knots`, `spline_knots`, `weights`,
/// `spline_verts` and `spline_normals`, each array as its `walk_data`
/// folds it — the length, and for a non-empty one the size, the
/// increment, the flags and the elements (`Array<Vector<float>>` one range
/// an element, `SimpleArray<float>` one for all). `None` for an arm whose
/// walk is not read.
///
/// The `calc_from_dir` arm (item 4495): `max_control_depth_ratio` 4.0, the
/// degree 2 and the depth `set_min_seg_length` wrote, its three control
/// points, no `knots`, `clamp_knots`' `0 0 0 1 1 1`. Every array's size is
/// the `Spline`'s own ([`Spline::caps`]), which a recycled one carries
/// from its last round.
fn spline_ranges(path: &Spline) -> Option<Vec<(&'static str, Vec<u8>)>> {
    let zero = Single::ZERO;
    let one = Single::from_bits(0x3f80_0000);
    let nuke;
    let arm = path.arm();
    let (ratio, degree, depth, ctl, weights, knots): ArmWalk = match &arm {
        SplineArm::Nuke { s, e } => {
            nuke = Spline::nuke_parts(*s, *e);
            (
                0,
                3,
                crate::air::MISSILE_FLIGHT as u16,
                &nuke.ctl,
                &nuke.weights,
                &nuke.knots,
            )
        }
        SplineArm::FromDir { ctl, depth } => (
            0x4080_0000,
            2,
            *depth,
            ctl,
            &[],
            &[zero, zero, zero, one, one, one],
        ),
        SplineArm::Unwalked => return None,
    };
    let caps = path.caps();
    let n = ctl.len();
    let (normals, length) = path.normals([ctl[n - 2], ctl[n - 1]]);
    let mut out = Vec::new();
    let mut head = Vec::with_capacity(36);
    for word in [
        3,                      // type: `generate_bspline`
        0x10,                   // flags: `Ammo::init`'s
        ratio,                  // max_control_depth_ratio
        length.bits() as i32,   // total_spline_length
        0,                      // last_knot
        0xbf80_0000_u32 as i32, // curr_dist, −1.0 (`Spline::clear`)
        0xbf80_0000_u32 as i32, // next_search_dist, −1.0
        -1,                     // search_scan
    ] {
        head.extend(word.to_le_bytes());
    }
    head.extend(degree.to_le_bytes());
    head.extend(depth.to_le_bytes());
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
    let verts = |out: &mut Vec<(&'static str, Vec<u8>)>,
                 name: [&'static str; 5],
                 vs: &[[Single; 3]],
                 cap: u32| {
        header(
            out,
            [name[0], name[1], name[2], name[3]],
            vs.len(),
            cap as usize,
        );
        for v in vs {
            out.push((
                name[4],
                v.iter().flat_map(|c| c.bits().to_le_bytes()).collect(),
            ));
        }
    };
    let floats = |out: &mut Vec<(&'static str, Vec<u8>)>,
                  name: [&'static str; 5],
                  fs: &[Single],
                  cap: u32| {
        header(
            out,
            [name[0], name[1], name[2], name[3]],
            fs.len(),
            cap as usize,
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
        ctl,
        caps.ctl,
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
        weights,
        caps.knots,
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
        knots,
        caps.spline_knots,
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
        caps.weights,
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
        caps.verts,
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
        caps.normals,
    );
    Some(out)
}

impl Sim {
    pub fn checksum_ammo(&self) -> Option<Line> {
        let mut ranges = Vec::with_capacity(self.combat.projectiles.len() * 3);
        for p in &self.combat.projectiles {
            // A falling aircraft's round (item 4523) is the same record,
            // `Ammo::init_crash@0067b800` writing `rolling` (`+0x5`), the
            // bank pair (`+0x5c`/`+0x60`, singles) and `start_roll_angle`
            // (`+0x64`). A guided round walks its spline (item 4495); one
            // without a spline is not read.
            if p.graphic.guided && p.spline.is_none() {
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
            // `+0x5`, the display roll (separate from flag 4): 0 from
            // `Ammo::init:830`, a crash's `r % 7 − 3`.
            body.push(p.crash.map_or(0, |c| c.rolling as i8 as u8));
            let (bank_dx, bank_dy, start_roll) = p.crash.map_or((0, 0, 0), |c| {
                (
                    Single::from_i32(c.bank.0).bits() as i32,
                    Single::from_i32(c.bank.1).bits() as i32,
                    c.start_roll,
                )
            });
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
                bank_dx,
                bank_dy,
                start_roll,
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
