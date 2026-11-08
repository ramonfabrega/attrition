//! What the camera's two checks share: running an oracle script under
//! `tools/emu`'s project, reading its rows, and scoring them against this
//! crate (`docs/RENDERER.md` §3.4).

use render::camera::{Camera, Mat4, Screen, world_to_screen};
use std::path::{Path, PathBuf};
use std::process::Command;

fn f(h: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(h, 16).expect("a float's bits"))
}

/// One camera state, the original's `to_screen_matrix` for it, and its
/// points: `x y z w h` and the original's `sx sy off`.
pub struct State {
    pub cam: Camera,
    pub screen: Screen,
    pub original: Mat4,
    pub points: Vec<[i32; 8]>,
}

/// The rows `camera_oracle.py` and `packet_points.py` print.
pub fn parse(rows: &str) -> Vec<State> {
    let mut out: Vec<State> = Vec::new();
    for line in rows.lines() {
        let t: Vec<&str> = line.split_whitespace().collect();
        match t.first() {
            Some(&"state") => {
                let v = &t[2..];
                out.push(State {
                    cam: Camera {
                        zoom_distance: f(v[0]),
                        camera_scale: f(v[1]),
                        tilt_angle: f(v[2]),
                        dir_angle: f(v[3]),
                        loc_x: f(v[4]),
                        loc_y: f(v[5]),
                        loc_x_adjust: f(v[6]),
                        loc_y_adjust: f(v[7]),
                        loc_z_adjust: f(v[8]),
                        angle: f(v[9]),
                        near_plane: f(v[10]),
                        far_plane: f(v[11]),
                        view_ratio: f(v[12]),
                        parallel: v[13] != "0",
                        parallel_distance: f(v[14]),
                    },
                    screen: Screen {
                        scaled_xs: v[15].parse().unwrap(),
                        scaled_ys: v[16].parse().unwrap(),
                        natural_ratio: f(v[17]),
                        console_y: v[18].parse().unwrap(),
                    },
                    original: Mat4::IDENTITY,
                    points: Vec::new(),
                });
            }
            Some(&"mat") => {
                let s = out.last_mut().expect("a state before its matrix");
                for (k, h) in t[2..18].iter().enumerate() {
                    s.original.0[k] = f(h);
                }
            }
            Some(&"pt") => {
                let n: Vec<i32> = t[2..]
                    .iter()
                    .filter(|w| **w != "->")
                    .map(|w| w.parse().unwrap())
                    .collect();
                out.last_mut()
                    .expect("a state before its points")
                    .points
                    .push(n.try_into().unwrap());
            }
            _ => {}
        }
    }
    out
}

/// `uv run --locked --project tools/emu python tools/render/<script> <args…>`,
/// or `None` (said on stderr) when `uv` cannot run.
pub fn oracle(script: &str, args: &[&Path], tail: &[String]) -> Option<String> {
    let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let out = Command::new("uv")
        .args(["run", "--locked", "--project"])
        .arg(root.join("tools/emu"))
        .arg("python")
        .arg(root.join("tools/render").join(script))
        .args(args)
        .args(tail)
        .output();
    match out {
        Err(e) => {
            eprintln!("skipping: uv not runnable ({e})");
            None
        }
        Ok(o) => {
            assert!(
                o.status.success(),
                "{script} failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            Some(String::from_utf8(o.stdout).expect("utf-8 rows"))
        }
    }
}

/// `$RON_INSTALL`, or `None` (said on stderr).
pub fn install() -> Option<PathBuf> {
    let i = std::env::var("RON_INSTALL")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    if i.is_none() {
        eprintln!(
            "skipping: set RON_INSTALL (the oracle runs the install's executable and d3dgl.dll)"
        );
    }
    i
}

/// The score of a set of states: how many are in register — every point
/// within one pixel on both axes and the off-screen flag equal — with the
/// points in register, the points in all, and the worst `to_screen` element
/// against the original's, relative. A state out of register is printed by
/// its parameters with each point it misses.
pub struct Score {
    pub states: usize,
    pub points: usize,
    pub of_points: usize,
    pub worst_matrix: f32,
}

pub fn in_register(states: &[State]) -> Score {
    let mut score = Score {
        states: 0,
        points: 0,
        of_points: 0,
        worst_matrix: 0.0,
    };
    for (i, s) in states.iter().enumerate() {
        let m = s.cam.to_screen_matrix();
        for (a, b) in m.0.iter().zip(&s.original.0) {
            score.worst_matrix = score.worst_matrix.max((a - b).abs() / (1.0 + b.abs()));
        }
        let mut misses = Vec::new();
        for p in &s.points {
            let size = (p[3] >= 1 && p[4] >= 1).then_some((p[3], p[4]));
            let got = world_to_screen(&m, &s.screen, p[0], p[1], p[2], size);
            score.of_points += 1;
            if (got.x - p[5]).abs() <= 1
                && (got.y - p[6]).abs() <= 1
                && got.off_screen == (p[7] != 0)
            {
                score.points += 1;
            } else {
                misses.push(format!(
                    "{p:?} -> ours ({}, {}, {})",
                    got.x, got.y, got.off_screen
                ));
            }
        }
        if misses.is_empty() {
            score.states += 1;
        } else {
            eprintln!(
                "state {i} out of register: {:?}\n  {}",
                s.cam,
                misses.join("\n  ")
            );
        }
    }
    score
}
