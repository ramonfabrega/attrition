//! The mountain templates — `effects_graphics.xml`'s `<MOUNTAINS>` and the
//! `TEMPLATE_TEX` image each one names (`docs/FORMATS.md`, "The mountain
//! templates").
//!
//! A placed mountain is a location and a template index; the generator
//! prints both (`GameLog::dump_mountains@0092fca0`, a `DUMP_ALL` block),
//! and the template is what says which tiles and which cells the range
//! holds. `MountainRange::init@008998b0` builds both lists from the
//! **alpha channel** of `TEMPLATE_TEX`, loaded 32-bit: a 256 × 256 image
//! where four pixels are a tile and sixteen a cell, centred on the
//! placed location. Nothing about the lists is authored as a list.
//!
//! - **The tiles** (`mount_tx`/`_ty`). Every fourth pixel from `x0 =
//!   (w/2) % 16` is a tile corner; a corner is set when its pixel's alpha
//!   is not zero, and a tile is the range's when all four of its corners
//!   are set. Row by row, which is the order the lists are appended in.
//! - **The solid cells** (`solid_mount_wx`/`_wy`). Each 16-pixel block
//!   whose far edge is still inside the image samples a 5 × 5 grid of
//!   pixels at 0, 4, 8, 12 and 16 — the far edge is the next block's near
//!   one — and the cell is solid when **more than fifteen** of the 25 are
//!   set. `init` also accepts all 25 set, which the count already covers.
//!
//! Both offsets are measured from the image centre, `(x / 16) − ((w/2 −
//! x0) / 16)` for a cell and the same over four for a tile.
//!
//! **Evidence.** run144's packet holds East Indies' eighteen placed ranges
//! and their 107 solid cells in three templates; these rules give every
//! one of them, and read the image with its first row at the top — the
//! file's own origin is bottom-left (`the_templates_give_the_packet_s_
//! solid_cells`). Great Lakes' range 6 is template 9, and its 244 tiles
//! are what run97 pinned the stand-in component at.

use std::path::Path;

use sim::gather::MountainTemplate;

use crate::Install;

/// A decoded TGA: its size and its pixels as `[b, g, r, a]`, **top row
/// first** whatever the file's origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tga {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[u8; 4]>,
}

impl Tga {
    /// The alpha at `(x, y)`, zero outside the image.
    pub fn alpha(&self, x: usize, y: usize) -> u8 {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x][3]
        } else {
            0
        }
    }
}

/// Reads a 32-bit true-colour TGA, raw (type 2) or run-length (type 10).
///
/// The header is eighteen bytes: an id length at 0, a colour-map type at
/// 1, the image type at 2, the width and height as `u16` at 12 and 14,
/// the bit depth at 16 and the descriptor at 17, whose bit 5 says the
/// first row stored is the top one. The shipped templates are type 10,
/// 256 × 256, 32 bits, descriptor `0x08` — eight alpha bits, bottom-left
/// origin. Anything else is refused rather than guessed at.
pub fn decode_tga(bytes: &[u8]) -> Option<Tga> {
    let head = bytes.get(..18)?;
    let (id_len, cmap, kind) = (usize::from(head[0]), head[1], head[2]);
    let width = usize::from(u16::from_le_bytes([head[12], head[13]]));
    let height = usize::from(u16::from_le_bytes([head[14], head[15]]));
    let (depth, desc) = (head[16], head[17]);
    if cmap != 0 || depth != 32 || !(kind == 2 || kind == 10) {
        return None;
    }
    let n = width * height;
    let mut at = 18 + id_len;
    let mut px: Vec<[u8; 4]> = Vec::with_capacity(n);
    let take = |at: usize| -> Option<[u8; 4]> { bytes.get(at..at + 4)?.try_into().ok() };
    while px.len() < n {
        if kind == 2 {
            px.push(take(at)?);
            at += 4;
            continue;
        }
        let c = *bytes.get(at)?;
        at += 1;
        let run = usize::from(c & 0x7f) + 1;
        if c & 0x80 != 0 {
            let v = take(at)?;
            at += 4;
            px.extend(std::iter::repeat_n(v, run));
        } else {
            for _ in 0..run {
                px.push(take(at)?);
                at += 4;
            }
        }
    }
    if px.len() != n {
        return None;
    }
    if desc & 0x20 == 0 {
        // Bottom-left origin: the file's first row is the image's last.
        px = px.chunks(width).rev().flatten().copied().collect();
    }
    Some(Tga {
        width,
        height,
        pixels: px,
    })
}

/// `MountainRange::init@008998b0`'s two lists from one template image.
pub fn template(t: &Tga) -> MountainTemplate {
    let (w, h) = (t.width as i32, t.height as i32);
    // `(w/2) % 16`: the image centre falls on a cell corner.
    let (x0, y0) = ((w / 2) % 16, (h / 2) % 16);
    let set = |x: i32, y: i32| x >= 0 && y >= 0 && t.alpha(x as usize, y as usize) != 0;
    let mut tiles = Vec::new();
    // The quad pass: rows bounded by the width and columns by the height,
    // as `init` writes them; the templates are square.
    let mut y = y0;
    while y < w {
        let mut x = x0;
        while x < h {
            if set(x, y) && set(x + 4, y) && set(x, y + 4) && set(x + 4, y + 4) {
                tiles.push((x / 4 - (w / 2 - x0) / 4, y / 4 - (h / 2 - y0) / 4));
            }
            x += 4;
        }
        y += 4;
    }
    let mut solid = Vec::new();
    let mut y = y0;
    while y + 16 < h {
        let mut x = x0;
        while x + 16 < w {
            let mut count = 0;
            for i in (0..=16).step_by(4) {
                for j in (0..=16).step_by(4) {
                    if set(x + i, y + j) {
                        count += 1;
                    }
                }
            }
            if count > 15 {
                solid.push((x / 16 - (w / 2 - x0) / 16, y / 16 - (h / 2 - y0) / 16));
            }
            x += 16;
        }
        y += 16;
    }
    MountainTemplate { tiles, solid }
}

/// Every `<MOUNTAIN>` under `effects_graphics.xml`'s `<MOUNTAINS>`, in file
/// order — which is the template index: `Mountains::add_range@008992b0`
/// hands out the first free of sixteen slots, one per element. Each is
/// its first child's `file`, `TEMPLATE_TEX`, which `Mountains::init
/// @0089ad70` passes to `MountainRange::init` as the image it reads
/// (the other two are the renderer's).
///
/// A template whose image this reader cannot find or decode is left
/// empty rather than dropped, so the indices stay the generator's. The
/// whole list is empty when the XML is unreadable, which is what a
/// tables-only load gets.
pub fn templates(install: &Install) -> Vec<MountainTemplate> {
    template_files(install)
        .iter()
        .map(|rel| {
            crate::artdata::resolve_art(install.root(), rel)
                .and_then(|p| std::fs::read(p).ok())
                .and_then(|b| decode_tga(&b))
                .map(|t| template(&t))
                .unwrap_or_default()
        })
        .collect()
}

/// Each `<MOUNTAIN>`'s `TEMPLATE_TEX` path as the XML writes it.
pub fn template_files(install: &Install) -> Vec<String> {
    let path = install.data("effects_graphics.xml");
    let Ok(text) = crate::read(&path) else {
        return Vec::new();
    };
    let Ok(doc) = crate::parse(&path, &text) else {
        return Vec::new();
    };
    let Some(list) = doc
        .root_element()
        .children()
        .find(|n| n.is_element() && n.tag_name().name() == "MOUNTAINS")
    else {
        return Vec::new();
    };
    list.children()
        .filter(|n| n.is_element() && n.tag_name().name() == "MOUNTAIN")
        .map(|m| {
            m.children()
                .find(|n| n.is_element())
                .and_then(|n| n.attribute("file"))
                .unwrap_or_default()
                .to_string()
        })
        .collect()
}

/// The decoded template image at `rel`, for the survey's format checks.
pub fn template_image(root: &Path, rel: &str) -> Option<(Tga, u8, u8)> {
    let bytes = std::fs::read(crate::artdata::resolve_art(root, rel)?).ok()?;
    let (kind, desc) = (*bytes.get(2)?, *bytes.get(17)?);
    Some((decode_tga(&bytes)?, kind, desc))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A TGA header for `w × h`, 32 bits, of `kind`, bottom-left origin.
    fn header(kind: u8, w: u16, h: u16, desc: u8) -> Vec<u8> {
        let mut b = vec![0u8; 18];
        b[2] = kind;
        b[12..14].copy_from_slice(&w.to_le_bytes());
        b[14..16].copy_from_slice(&h.to_le_bytes());
        b[16] = 32;
        b[17] = desc;
        b
    }

    #[test]
    fn a_run_length_image_decodes_top_row_first() {
        // 2 × 2, bottom-left origin: the file's first row is the bottom.
        let mut b = header(10, 2, 2, 0x08);
        // A run of two opaque pixels (the bottom row), then two raw ones.
        b.extend([0x81, 1, 2, 3, 255]);
        b.extend([0x01, 9, 9, 9, 0, 7, 7, 7, 128]);
        let t = decode_tga(&b).unwrap();
        assert_eq!((t.width, t.height), (2, 2));
        assert_eq!(t.alpha(0, 0), 0, "the stored second row is the top");
        assert_eq!(t.alpha(1, 0), 128);
        assert_eq!(t.alpha(0, 1), 255);
        assert_eq!(t.alpha(1, 1), 255);
    }

    #[test]
    fn a_top_left_raw_image_is_not_flipped() {
        let mut b = header(2, 1, 2, 0x28);
        b.extend([0, 0, 0, 10, 0, 0, 0, 20]);
        let t = decode_tga(&b).unwrap();
        assert_eq!((t.alpha(0, 0), t.alpha(0, 1)), (10, 20));
    }

    #[test]
    fn a_short_or_foreign_image_is_refused() {
        let mut b = header(10, 2, 2, 0x08);
        b.extend([0x81, 1, 2, 3, 255]);
        assert_eq!(decode_tga(&b), None, "two pixels of four");
        let mut c = header(2, 1, 1, 0x08);
        c[16] = 24;
        c.extend([0, 0, 0]);
        assert_eq!(decode_tga(&c), None, "24 bits is not what init loads");
    }

    /// A 64 × 64 image, alpha set on `[x0, x1] × [y0, y1]` (pixels).
    fn image(x0: usize, x1: usize, y0: usize, y1: usize) -> Tga {
        let (w, h) = (64, 64);
        let mut pixels = vec![[0u8; 4]; w * h];
        for y in y0..=y1 {
            for x in x0..=x1 {
                pixels[y * w + x][3] = 255;
            }
        }
        Tga {
            width: w,
            height: h,
            pixels,
        }
    }

    #[test]
    fn a_cell_is_solid_on_sixteen_of_its_twenty_five_samples() {
        // The centre is pixel 32: cell 0 is pixels 32..=48. Set its first
        // four sample columns and all five rows: 20 of 25.
        let t = template(&image(32, 44, 32, 48));
        assert_eq!(t.solid, vec![(0, 0)]);
        // Three columns: 15 of 25, not solid.
        let t = template(&image(32, 40, 32, 48));
        assert!(t.solid.is_empty(), "{:?}", t.solid);
        // The far edge is the next block's near one: sixteen samples of
        // four columns by four rows fill cell (0, 0) and nothing else.
        let t = template(&image(32, 44, 32, 44));
        assert_eq!(t.solid, vec![(0, 0)]);
    }

    #[test]
    fn a_tile_needs_all_four_corners() {
        // Corners at pixels 32 and 36 in both axes: one tile, (0, 0).
        let t = template(&image(32, 36, 32, 36));
        assert_eq!(t.tiles, vec![(0, 0)]);
        // Corners 28..=36 across and 32..=36 down: two tiles, in a row.
        let t = template(&image(28, 36, 32, 36));
        assert_eq!(t.tiles, vec![(-1, 0), (0, 0)]);
    }
}
