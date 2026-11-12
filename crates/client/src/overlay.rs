//! **The debug overlay** (`docs/RENDERER.md` §7.2 item 8, the seventy-fifth
//! pass's ruling): the frame number, the clock and the camera, and nothing
//! else — every drawn element that is not a piece or the terrain is scope
//! the counter cannot see. Drawn as pixel quads from a 5×7 font kept here,
//! so the overlay costs no font dependency.

/// A glyph: seven rows of five bits, the high bit leftmost.
fn glyph(c: char) -> [u8; 7] {
    match c.to_ascii_uppercase() {
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1f, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x1e, 0x01, 0x01, 0x11, 0x0e],
        '6' => [0x06, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x0c],
        'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        'C' => [0x0e, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0e],
        'D' => [0x1c, 0x12, 0x11, 0x11, 0x11, 0x12, 0x1c],
        'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        'G' => [0x0e, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0f],
        'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        'I' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0c],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11],
        'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0a],
        'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x11, 0x0a, 0x04, 0x04, 0x04],
        'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        '.' => [0, 0, 0, 0, 0, 0x0c, 0x0c],
        ':' => [0, 0x0c, 0x0c, 0, 0x0c, 0x0c, 0],
        '-' => [0, 0, 0, 0x1f, 0, 0, 0],
        '/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        '(' => [0x02, 0x04, 0x08, 0x08, 0x08, 0x04, 0x02],
        ')' => [0x08, 0x04, 0x02, 0x02, 0x02, 0x04, 0x08],
        ',' => [0, 0, 0, 0, 0x0c, 0x04, 0x08],
        // The lobby page's (item 4184): the categories' titles and names.
        '>' => [0x08, 0x04, 0x02, 0x01, 0x02, 0x04, 0x08],
        '<' => [0x02, 0x04, 0x08, 0x10, 0x08, 0x04, 0x02],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0, 0x04],
        '?' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0, 0x04],
        '&' => [0x0c, 0x12, 0x14, 0x08, 0x15, 0x12, 0x0d],
        '%' => [0x18, 0x19, 0x02, 0x04, 0x08, 0x13, 0x03],
        '+' => [0, 0x04, 0x04, 0x1f, 0x04, 0x04, 0],
        '=' => [0, 0, 0x1f, 0, 0x1f, 0, 0],
        '\'' => [0x04, 0x04, 0x08, 0, 0, 0, 0],
        '"' => [0x0a, 0x0a, 0, 0, 0, 0, 0],
        '#' => [0x0a, 0x0a, 0x1f, 0x0a, 0x1f, 0x0a, 0x0a],
        '_' => [0, 0, 0, 0, 0, 0, 0x1f],
        ';' => [0, 0x0c, 0x0c, 0, 0x0c, 0x04, 0x08],
        _ => [0; 7],
    }
}

/// Whether the font draws `c` (a space draws nothing, and has one).
#[cfg(test)]
pub fn has_glyph(c: char) -> bool {
    c == ' ' || glyph(c) != [0; 7]
}

/// The overlay's text, one line each.
pub struct Lines {
    pub frame: i64,
    pub frame_ms: f64,
    pub camera: render::Camera,
}

impl Lines {
    pub fn text(&self) -> Vec<String> {
        let c = &self.camera;
        vec![
            format!("FRAME {}", self.frame),
            format!("CLOCK {:.1} MS", self.frame_ms),
            format!(
                "CAMERA {:.0},{:.0} TILT {:.0} DIR {:.0} PDIST {:.0}",
                c.loc_x, c.loc_y, c.tilt_angle, c.dir_angle, c.parallel_distance
            ),
        ]
    }
}

/// Quads in normalised device coordinates on a `size` screen, placed in
/// pixels: the debug overlay's, and the HUD's (item 4197, `docs/HUD.md`).
pub struct Painter {
    size: (i32, i32),
    pub pos: Vec<[f32; 3]>,
    pub col: Vec<[f32; 3]>,
}

/// The overlay's text colour.
pub const INK: [f32; 3] = [0.95, 0.92, 0.55];
/// The overlay's panel colour.
pub const PANEL: [f32; 3] = [0.02, 0.02, 0.03];

impl Painter {
    pub fn new(size: (i32, i32)) -> Painter {
        Painter {
            size,
            pos: Vec::new(),
            col: Vec::new(),
        }
    }

    /// A `w` by `h` rectangle with its top-left corner at pixel `(x, y)`.
    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: [f32; 3]) {
        let (sw, sh) = (self.size.0 as f32, self.size.1 as f32);
        let (x0, y0) = (x as f32 / sw * 2.0 - 1.0, 1.0 - y as f32 / sh * 2.0);
        let (x1, y1) = (
            (x + w) as f32 / sw * 2.0 - 1.0,
            1.0 - (y + h) as f32 / sh * 2.0,
        );
        for p in [[x0, y0], [x1, y0], [x0, y1], [x1, y0], [x1, y1], [x0, y1]] {
            self.pos.push([p[0], p[1], 0.0]);
            self.col.push(c);
        }
    }

    /// One line of text, its first glyph's top-left at `(x, y)`, each lit
    /// font pixel a `scale`-pixel square.
    pub fn text(&mut self, x: i32, y: i32, line: &str, scale: i32, c: [f32; 3]) {
        for (k, ch) in line.chars().enumerate() {
            let g = glyph(ch);
            let ox = x + k as i32 * advance(scale);
            for (gy, bits) in g.iter().enumerate() {
                for gx in 0..5 {
                    if bits & (0x10 >> gx) != 0 {
                        self.rect(ox + gx * scale, y + gy as i32 * scale, scale, scale, c);
                    }
                }
            }
        }
    }
}

/// A glyph's advance, in pixels at `scale`.
pub fn advance(scale: i32) -> i32 {
    6 * scale
}

/// A line's height, in pixels at `scale`.
pub fn line_height(scale: i32) -> i32 {
    9 * scale
}

/// The overlay as triangles in normalised device coordinates on a
/// `size` screen: a dark panel in the top-left corner and the text over
/// it, each lit font pixel a `scale`-pixel square.
pub fn triangles(lines: &[String], size: (i32, i32), scale: i32) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let mut p = Painter::new(size);
    let (cw, ch) = (advance(scale), line_height(scale));
    let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as i32;
    let (px, py) = (8, 8);
    p.rect(
        px,
        py,
        longest * cw + 2 * scale * 2,
        lines.len() as i32 * ch + 2 * scale * 2,
        PANEL,
    );
    for (row, line) in lines.iter().enumerate() {
        p.text(
            px + 2 * scale,
            py + 2 * scale + row as i32 * ch,
            line,
            scale,
            INK,
        );
    }
    (p.pos, p.col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_character_the_overlay_prints_has_a_glyph() {
        let camera = render::Camera {
            zoom_distance: 1.0,
            camera_scale: 1.0,
            tilt_angle: 60.0,
            dir_angle: 45.0,
            loc_x: 3168.0,
            loc_y: 29744.7,
            loc_x_adjust: 0.0,
            loc_y_adjust: 0.0,
            loc_z_adjust: 0.0,
            angle: 45.0,
            near_plane: 1.0,
            far_plane: 2.0,
            view_ratio: 0.5625,
            parallel: true,
            parallel_distance: 10429.4,
        };
        let lines = Lines {
            frame: 3294,
            frame_ms: 12.5,
            camera,
        }
        .text();
        for c in lines.iter().flat_map(|l| l.chars()).filter(|c| *c != ' ') {
            assert_ne!(glyph(c), [0; 7], "no glyph for {c:?}");
        }
        let (pos, col) = triangles(&lines, (1920, 1080), 2);
        assert_eq!(pos.len(), col.len());
        assert!(pos.len() > 6, "a panel and some text");
        assert!(
            pos.iter()
                .all(|p| (-1.0..=1.0).contains(&p[0]) && (-1.0..=1.0).contains(&p[1])),
            "on screen"
        );
    }
}
