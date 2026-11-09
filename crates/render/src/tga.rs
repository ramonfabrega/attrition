//! The install's Truevision `.tga` images (`docs/FORMATS.md`, "The gaia
//! pieces and the building pieces", the images).
//!
//! An eighteen-byte header, an optional image id, the pixels, and an
//! optional twenty-six-byte version-2 footer ending
//! `TRUEVISION-XFILE.\0`. Every one of the install's 2,901 under `art/`
//! and `terrain art/` is one of three kinds — uncompressed true colour
//! (type 2), uncompressed grey (3) or run-length true colour (10) — with
//! no colour map, at 8, 24 or 32 bits, and ends exactly at its last pixel
//! or at its footer (item 4037). The reader holds every file to that, which
//! is what makes it self-checking: a run that overruns the image, pixels
//! that stop short of where the file says they end, bytes left between the
//! pixels and the footer, or a footer pointing outside the file are each a
//! named refusal rather than a picture.
//!
//! The pixels come out as RGBA8, rows top-down, whatever the file's origin.

use std::fmt;

/// A decoded image: `width · height` pixels, RGBA8, the top row first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    /// The file's image type: 2, 3 or 10.
    pub kind: u8,
    /// The file's bits per pixel: 8, 24 or 32.
    pub bits: u8,
    /// Whether the file ends in a version-2 footer.
    pub footer: bool,
    pub rgba: Vec<u8>,
}

/// Why a file is not one this reader understands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// Fewer than the header's eighteen bytes.
    Short { len: usize },
    /// An image type, colour map or depth outside the install's.
    Unsupported { kind: u8, colour_map: u8, bits: u8 },
    /// A descriptor whose alpha bits disagree with the depth, or that
    /// stores its columns right to left.
    Descriptor { descriptor: u8, bits: u8 },
    /// A zero width or height.
    Empty,
    /// The pixels run past the bytes there are, at `at`.
    Truncated { at: usize },
    /// A run-length packet that runs past the image's last pixel.
    RunPastImage { at: usize },
    /// Bytes after the pixels that are not exactly a version-2 footer.
    Trailing { pixels_end: usize, len: usize },
    /// A footer whose extension or developer area is not inside the gap
    /// between the pixels and the footer.
    FooterOffset { offset: u32 },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Short { len } => write!(f, "{len} bytes, under a header"),
            Self::Unsupported {
                kind,
                colour_map,
                bits,
            } => write!(f, "type {kind}, colour map {colour_map}, {bits} bits"),
            Self::Descriptor { descriptor, bits } => {
                write!(f, "descriptor {descriptor:#04x} at {bits} bits")
            }
            Self::Empty => write!(f, "no pixels"),
            Self::Truncated { at } => write!(f, "pixels truncated at {at}"),
            Self::RunPastImage { at } => write!(f, "a run past the image at {at}"),
            Self::Trailing { pixels_end, len } => {
                write!(f, "pixels end at {pixels_end} of {len}, not at a footer")
            }
            Self::FooterOffset { offset } => write!(f, "footer offset {offset} outside the file"),
        }
    }
}

impl std::error::Error for DecodeError {}

const FOOTER: &[u8; 18] = b"TRUEVISION-XFILE.\0";

fn u16_at(b: &[u8], at: usize) -> usize {
    u16::from_le_bytes([b[at], b[at + 1]]) as usize
}

/// One stored pixel, BGR(A) or grey, as RGBA.
fn pixel(px: &[u8]) -> [u8; 4] {
    match px.len() {
        1 => [px[0], px[0], px[0], 0xff],
        3 => [px[2], px[1], px[0], 0xff],
        _ => [px[2], px[1], px[0], px[3]],
    }
}

/// Decode a whole `.tga`, refusing anything the install does not ship.
pub fn decode(bytes: &[u8]) -> Result<Image, DecodeError> {
    if bytes.len() < 18 {
        return Err(DecodeError::Short { len: bytes.len() });
    }
    let (id_len, colour_map, kind) = (bytes[0] as usize, bytes[1], bytes[2]);
    let (width, height, bits, descriptor) =
        (u16_at(bytes, 12), u16_at(bytes, 14), bytes[16], bytes[17]);
    let grey = kind == 3;
    let depth_ok = if grey {
        bits == 8
    } else {
        bits == 24 || bits == 32
    };
    if colour_map != 0 || !matches!(kind, 2 | 3 | 10) || !depth_ok {
        return Err(DecodeError::Unsupported {
            kind,
            colour_map,
            bits,
        });
    }
    let alpha = descriptor & 0x0f;
    let alpha_ok = match bits {
        32 => alpha == 8,
        8 => alpha == 8 || alpha == 0,
        _ => alpha == 0,
    };
    if !alpha_ok || descriptor & 0x10 != 0 || descriptor & 0xc0 != 0 {
        return Err(DecodeError::Descriptor { descriptor, bits });
    }
    if width == 0 || height == 0 {
        return Err(DecodeError::Empty);
    }
    let stride = bits as usize / 8;
    let count = width * height;
    let mut at = 18 + id_len;
    let mut stored: Vec<[u8; 4]> = Vec::with_capacity(count);
    if kind == 10 {
        while stored.len() < count {
            let head = *bytes.get(at).ok_or(DecodeError::Truncated { at })?;
            let n = (head & 0x7f) as usize + 1;
            if stored.len() + n > count {
                return Err(DecodeError::RunPastImage { at });
            }
            at += 1;
            if head & 0x80 != 0 {
                let px = bytes
                    .get(at..at + stride)
                    .ok_or(DecodeError::Truncated { at })?;
                stored.extend(std::iter::repeat_n(pixel(px), n));
                at += stride;
            } else {
                let run = bytes
                    .get(at..at + n * stride)
                    .ok_or(DecodeError::Truncated { at })?;
                stored.extend(run.chunks_exact(stride).map(pixel));
                at += n * stride;
            }
        }
    } else {
        let run = bytes
            .get(at..at + count * stride)
            .ok_or(DecodeError::Truncated { at: bytes.len() })?;
        stored.extend(run.chunks_exact(stride).map(pixel));
        at += count * stride;
    }
    let footer = match bytes.len() - at {
        0 => false,
        26 if bytes.ends_with(FOOTER) => {
            let foot = bytes.len() - 26;
            for o in [&bytes[foot..foot + 4], &bytes[foot + 4..foot + 8]] {
                let offset = u32::from_le_bytes([o[0], o[1], o[2], o[3]]);
                if offset != 0 && !(at..foot).contains(&(offset as usize)) {
                    return Err(DecodeError::FooterOffset { offset });
                }
            }
            true
        }
        _ => {
            return Err(DecodeError::Trailing {
                pixels_end: at,
                len: bytes.len(),
            });
        }
    };
    // Bit 5 set is a top-left origin; clear, the file's first row is the
    // bottom one.
    let top_down = descriptor & 0x20 != 0;
    let mut rgba = Vec::with_capacity(count * 4);
    for row in 0..height {
        let src = if top_down { row } else { height - 1 - row };
        for px in &stored[src * width..(src + 1) * width] {
            rgba.extend_from_slice(px);
        }
    }
    Ok(Image {
        width,
        height,
        kind,
        bits,
        footer,
        rgba,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2×2 image in the install's commonest shape, built by hand: 32-bit
    /// BGRA, bottom-left origin (descriptor 8), the given body and footer.
    fn file(kind: u8, body: &[u8], footer: bool) -> Vec<u8> {
        let mut b = vec![0, 0, kind, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 2, 0, 32, 8];
        b.extend_from_slice(body);
        if footer {
            b.extend_from_slice(&[0; 8]);
            b.extend_from_slice(FOOTER);
        }
        b
    }

    const BGRA: [[u8; 4]; 4] = [
        [1, 2, 3, 4],
        [5, 6, 7, 8],
        [9, 10, 11, 12],
        [13, 14, 15, 16],
    ];

    #[test]
    fn an_uncompressed_image_comes_out_rgba_top_row_first() {
        let body: Vec<u8> = BGRA.concat();
        let img = decode(&file(2, &body, true)).unwrap();
        assert_eq!((img.width, img.height, img.footer), (2, 2, true));
        // The file's first row is the bottom one: pixels 2 and 3 lead.
        assert_eq!(
            img.rgba,
            [11, 10, 9, 12, 15, 14, 13, 16, 3, 2, 1, 4, 7, 6, 5, 8]
        );
    }

    #[test]
    fn a_run_length_image_decodes_to_the_same_pixels() {
        let raw = decode(&file(2, &[[1, 2, 3, 4]; 4].concat(), false)).unwrap();
        let mut body = vec![0x83];
        body.extend_from_slice(&[1, 2, 3, 4]);
        let rle = decode(&file(10, &body, false)).unwrap();
        assert_eq!(rle.rgba, raw.rgba);
        // A raw packet of two, then a repeat of two.
        let mut body = vec![0x01, 1, 2, 3, 4, 1, 2, 3, 4, 0x81, 1, 2, 3, 4];
        assert_eq!(decode(&file(10, &body, true)).unwrap().rgba, raw.rgba);
        // Made to fail: a run of three where two pixels remain.
        body[9] = 0x82;
        assert_eq!(
            decode(&file(10, &body, true)),
            Err(DecodeError::RunPastImage { at: 18 + 9 })
        );
    }

    #[test]
    fn a_file_that_does_not_end_at_its_pixels_or_footer_is_refused() {
        let mut body: Vec<u8> = BGRA.concat();
        assert!(decode(&file(2, &body, false)).is_ok());
        body.push(0);
        assert_eq!(
            decode(&file(2, &body, false)),
            Err(DecodeError::Trailing {
                pixels_end: 34,
                len: 35
            })
        );
        body.truncate(15);
        assert_eq!(
            decode(&file(2, &body, false)),
            Err(DecodeError::Truncated { at: 33 })
        );
        let mut colour_mapped = file(1, &BGRA.concat(), false);
        colour_mapped[1] = 1;
        assert!(matches!(
            decode(&colour_mapped),
            Err(DecodeError::Unsupported { kind: 1, .. })
        ));
    }
}
