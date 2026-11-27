//! **The military tally** (item 4761, `docs/RENDERER.md` §6.23): the
//! `StatWin`'s military page, the right-aligned lines above the bar's right
//! end on run1814b (`MILITARY`, `Scout: 1`, `Cities: 1`).
//!
//! `StatWin::init@007f0730` reads `<STATWIN name="statwin">` (500 × 400 at
//! (0, 0)), sets its bottom to `scaled_screen_ys − 0xc4` and moves its right
//! edge to the console's width: the window is x `[W − 500, W)`, y `[0,
//! H − 196)`. `StatWin::on_redraw@007f0e70`, with the profile's `prefs & 4`
//! set and a `stat_mode` other than 0, calls `setup_entries@007ed7b0` and
//! walks its lines **from the last to the first**: its pen starts at the
//! window's height less 3, and each line raises it by its face's height
//! before `Buffer::write_right_l` ends the line at the window's width less
//! 3, white (`add_line@007f0660` stores `WHITE` whatever colour it is
//! passed) over a black shadow. The face is `FontFamily::get(fonts,
//! "data_size", 12, −1, 1)` (`int_str_array +0x1e618`), `fonts.xml`'s
//! `data_size` lookup, 14 bold at 1280: the first such entry is
//! `georgiab.ttf`, the banner caption's ([`super::banner::CAPTION_FACE`]).
//!
//! `stat_mode` 1 is the military page (`setup_entries`):
//! 1. `MILITARY` (`loc_str_array +0x10f68`, `translated_strings.xml`'s
//!    3474th);
//! 2. per unit type in type order, the leader's count of it, where it is
//!    not zero and the type is no worker (`TypeData::is_worker`), no
//!    `is(CARA)`, no merchant and no `is(FISHERMEN)`: the type's name,
//!    `": "` (`+0x10c20`) and the count;
//! 3. the barracks', stable's, siege factory's, dock's, airbase's, missile
//!    silo's, tower's and fort's counts at the leader's current upgrade of
//!    each, where not zero (not built here: 4761a);
//! 4. `Cities` (`+0x10f7c`), `": "` and the leader's `city_num`, where not
//!    zero;
//! 5. the lookouts' count, likewise (4761a).

use crate::selection::{PlateFace, Texels};

/// `<STATWIN>`'s width in `interface.xml`.
pub const WIDTH: i32 = 500;

/// `StatWin::init`'s bottom: `scaled_screen_ys − 0xc4`.
pub const BOTTOM_MARGIN: i32 = 0xc4;

/// `on_redraw`'s inset from the window's width and height.
pub const INSET: i32 = 3;

/// The military page's heading (`loc_str_array +0x10f68`).
pub const HEADING: &str = "MILITARY";

/// `TypeIndex::CARA` and `FISHERMEN`, the two `is` tests the unit loop
/// makes (`vtable +0x60` with `strict` 0).
const CARA: i32 = 0x3b;
const FISHERMEN: i32 = 0x13d;

/// **`setup_entries`' military page** for `who` in `scene`, the lines in
/// order: the heading, each held unit type's `name: count` in type order,
/// and the cities. `names` is the unit types' `NAME`s by `ty`.
///
/// The unit count is the leader's per-type count (`leader +0x570e`, a
/// `ushort` per type from 50); ours counts `who`'s living units on the map
/// and in its buildings by `type_index`. `is(CARA)` and `is(FISHERMEN)` are
/// lineage tests; ours compares the index (4761b).
pub fn military_lines(
    scene: &sim::observe::Scene,
    names: &[String],
    who: sim::Player,
    cities: i32,
) -> Vec<String> {
    let mut held: std::collections::BTreeMap<i32, (Option<usize>, i32)> = Default::default();
    for u in scene
        .units
        .iter()
        .filter(|u| u.owner == who && u.health > 0)
    {
        let e = held.entry(u.type_index).or_insert((u.ty, 0));
        e.1 += 1;
    }
    for g in scene.garrisons.iter().filter(|g| g.owner == who) {
        held.entry(g.type_index).or_insert((None, 0)).1 += 1;
    }
    let mut v = vec![HEADING.to_string()];
    for (ti, (ty, n)) in held {
        if n == 0
            || (0x32..=0x35).contains(&ti)
            || ti == CARA
            || matches!(ti, 0x3d | 0x3e | 400)
            || ti == FISHERMEN
        {
            continue;
        }
        let Some(name) = ty.and_then(|t| names.get(t)) else {
            continue;
        };
        v.push(format!("{name}: {n}"));
    }
    if cities != 0 {
        v.push(format!("Cities: {cities}"));
    }
    v
}

/// **The window's corner** on a screen `size`: x `W − 500`, y 0.
pub fn origin(size: (i32, i32)) -> (i32, i32) {
    (size.0 - WIDTH, 0)
}

/// **The window written**: `lines` bottom-up from `height − 3`, each
/// ending at `width − 3`, white over a black shadow one pixel right and one
/// down, on a clear window `WIDTH` × `H − 196`.
pub fn compose(size: (i32, i32), face: &PlateFace, lines: &[String]) -> Texels {
    let (w, h) = (WIDTH, (size.1 - BOTTOM_MARGIN).max(0));
    let mut t = Texels {
        w: w as usize,
        h: h as usize,
        rgba: vec![0; (w * h * 4) as usize],
    };
    let mut y = h - INSET;
    for line in lines.iter().rev() {
        y -= face.height();
        write_right(&mut t, face, w - INSET, y, line);
    }
    t
}

/// **`Buffer::write_right_l`**: `line` ending at `x`, its cells' top at
/// `y`, the black shadow first at (+1, +1), then white.
fn write_right(win: &mut Texels, face: &PlateFace, x: i32, y: i32, line: &str) {
    let Some(tw) = face.text_width(line) else {
        return;
    };
    let cells: Vec<(i32, crate::selection::GlyphCell)> = line
        .chars()
        .scan(x - tw, |p, c| {
            let at = *p;
            *p += face.advance(c).unwrap_or(0);
            Some(face.cell(c).map(|cell| (at, cell)))
        })
        .flatten()
        .collect();
    let h = face.height();
    for (ink, (dx, dy)) in [([0u8, 0, 0], (1, 1)), ([255, 255, 255], (0, 0))] {
        for (at, cell) in &cells {
            for v in 0..h {
                for u in 0..cell.w as i32 {
                    let a = cell.cover[(v * cell.w as i32 + u) as usize];
                    let (px, py) = (at + u + dx, y + v + dy);
                    if a == 0
                        || !(0..win.w as i32).contains(&px)
                        || !(0..win.h as i32).contains(&py)
                    {
                        continue;
                    }
                    let i = (py as usize * win.w + px as usize) * 4;
                    super::left::over(&mut win.rgba[i..i + 4], [ink[0], ink[1], ink[2], a]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page's filters on a hand-built scene's worth of type indices:
    /// the worker and the merchant go, the scout stays, and the cities
    /// follow.
    #[test]
    fn the_page_skips_the_civilians() {
        let names = vec!["Citizen".to_string(), "Scout".to_string()];
        let scene = sim::observe::Scene::default();
        assert_eq!(military_lines(&scene, &names, 1, 0), vec!["MILITARY"]);
        assert_eq!(
            military_lines(&scene, &names, 1, 2),
            vec!["MILITARY", "Cities: 2"]
        );
        assert_eq!(origin((1920, 1080)), (1420, 0));
    }
}
