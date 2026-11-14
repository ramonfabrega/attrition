//! The surf's section models, `.mot` (`docs/FORMATS.md`, "The surf's
//! section models"; item 4235).
//!
//! Written from reading `MOTMethod::load@009164e0`, `load_mot@00914a70`,
//! `load_anim_header@00914970` and `load_frames@009146e0`, which fill an
//! `AnimMeshPacket` (the PDB names its fields), and
//! `SurfWaveSection::init@0089e8f0`, the one consumer, which keeps it.
//!
//! ```text
//! id 0  file                       num_chunks = 1 + frames
//!   id 11 header    u32 num_frames, u32 n, n × u16 face indices
//!   id 12 frame                    num_chunks 2
//!     id 2  vertices   u32 n, n × {f32 x, y, z}
//!     id 4  texcoords  u32 n, n × {f32 u, v}
//!   id 12 frame …
//! ```
//!
//! **One header, then the frames.** `load_frames` refuses a frame before
//! the header has made the packet, and `SurfWaveSection::init` keeps the
//! section only when `num_frames`, the vertex arrays' count and the
//! texcoord arrays' count are one number, so this reader holds a file to
//! all three. A vertex is three floats here, not the `.BH3`'s four.
//!
//! **What the section keeps** is less than the file: each frame's
//! vertices as `(3x, 3y)` — the loop at `0x89eb6d` multiplies both by the
//! `.rdata` float at `0x00b695c8`, 3.0, and steps 12 bytes — with `z`
//! dropped; each frame's texcoords whole; and the face list. The draw
//! places the section at z 0 (`SurfWaveSection::draw@0089e360`).

use crate::bh3::array;
use crate::chunk::{self, DecodeError, Reason, err, f32_at, u32_at};

/// `SurfWaveSection::init`'s scale on a vertex's `x` and `y`: the float
/// at `0x00b695c8`.
pub const SECTION_SCALE: f32 = 3.0;

/// A decoded section model: an `AnimMeshPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct Mot {
    /// `AnimMeshPacket::verts_list` and `texs_list`, a frame each.
    pub frames: Vec<Frame>,
    /// `AnimMeshPacket::faces`: whole triangles, every index a vertex.
    pub faces: Vec<u16>,
}

/// One keyframe of the shape.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub verts: Vec<[f32; 3]>,
    pub texs: Vec<[f32; 2]>,
}

impl Mot {
    /// The vertex count, the same in every frame.
    pub fn verts(&self) -> usize {
        self.frames[0].verts.len()
    }

    /// Frame `f`'s vertices as the section keeps them: `(3x, 3y)`.
    pub fn section_verts(&self, f: usize) -> Vec<[f32; 2]> {
        self.frames[f]
            .verts
            .iter()
            .map(|v| [v[0] * SECTION_SCALE, v[1] * SECTION_SCALE])
            .collect()
    }

    /// Whether every frame's texcoords equal the first frame's — the test
    /// `SurfWaveSection::init` runs when its `world_tex_coords` argument
    /// is 0, setting the field to −1 when they do and leaving 0 when not.
    pub fn texcoords_static(&self) -> bool {
        self.frames.iter().all(|f| f.texs == self.frames[0].texs)
    }
}

/// Decode a `.mot` file's bytes.
pub fn decode(bytes: &[u8]) -> Result<Mot, DecodeError> {
    let file = chunk::file(bytes)?;
    let kids = chunk::children(bytes, &file, file.body(), 0)?;
    let Some((head, rest)) = kids.split_first() else {
        return err(file.at, Reason::Missing { id: 11 });
    };
    if head.id != 11 || head.num_chunks != 0 {
        return err(
            head.at,
            Reason::UnknownId {
                id: head.id,
                parent: 0,
            },
        );
    }
    let num_frames = u32_at(bytes, head.body())?;
    let n = u32_at(bytes, head.body() + 4)?;
    let size = (head.end - head.at) as u32;
    if 16 + 2 * n as usize != head.end - head.at {
        return err(head.at, Reason::HeaderSize { indices: n, size });
    }
    let faces: Vec<u16> = bytes[head.body() + 8..head.end]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&b| u16::from_le_bytes(b))
        .collect();
    if !faces.len().is_multiple_of(3) {
        return err(
            head.at,
            Reason::PartialTriangle {
                indices: faces.len(),
            },
        );
    }
    if rest.len() != num_frames as usize {
        return err(
            head.at,
            Reason::FrameCount {
                declared: num_frames,
                found: rest.len(),
            },
        );
    }
    let mut frames = Vec::with_capacity(rest.len());
    for f in rest {
        if f.id != 12 {
            return err(
                f.at,
                Reason::UnknownId {
                    id: f.id,
                    parent: 0,
                },
            );
        }
        let parts = chunk::children(bytes, f, f.body(), 0)?;
        let (Some(v), Some(t)) = (parts.first(), parts.get(1)) else {
            return err(f.at, Reason::Missing { id: 2 });
        };
        if (v.id, t.id) != (2, 4) || v.num_chunks != 0 || t.num_chunks != 0 {
            let bad = if v.id != 2 { v } else { t };
            return err(
                bad.at,
                Reason::UnknownId {
                    id: bad.id,
                    parent: 12,
                },
            );
        }
        let (nv, va) = array(bytes, v, 12)?;
        let (nt, ta) = array(bytes, t, 8)?;
        if nt != nv {
            return err(
                t.at,
                Reason::CountMismatch {
                    id: 4,
                    count: nt,
                    verts: nv,
                },
            );
        }
        let mut verts = Vec::with_capacity(nv);
        for k in 0..nv {
            let at = va + 12 * k;
            verts.push([
                f32_at(bytes, at)?,
                f32_at(bytes, at + 4)?,
                f32_at(bytes, at + 8)?,
            ]);
        }
        let mut texs = Vec::with_capacity(nt);
        for k in 0..nt {
            let at = ta + 8 * k;
            texs.push([f32_at(bytes, at)?, f32_at(bytes, at + 4)?]);
        }
        frames.push(Frame { verts, texs });
    }
    let Some(first) = frames.first() else {
        return err(file.at, Reason::Missing { id: 12 });
    };
    let nv = first.verts.len();
    for (f, c) in frames.iter().zip(rest) {
        if f.verts.len() != nv {
            return err(
                c.at,
                Reason::CountMismatch {
                    id: 2,
                    count: f.verts.len(),
                    verts: nv,
                },
            );
        }
    }
    if let Some(&index) = faces.iter().find(|&&i| usize::from(i) >= nv) {
        return err(head.at, Reason::IndexOutOfRange { index, verts: nv });
    }
    Ok(Mot { frames, faces })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(id: u16, n: u16, body: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&(8 + body.len() as u32).to_le_bytes());
        v.extend_from_slice(&id.to_le_bytes());
        v.extend_from_slice(&n.to_le_bytes());
        v.extend_from_slice(body);
        v
    }

    fn header(frames: u32, faces: &[u16]) -> Vec<u8> {
        let mut b = frames.to_le_bytes().to_vec();
        b.extend_from_slice(&(faces.len() as u32).to_le_bytes());
        for i in faces {
            b.extend_from_slice(&i.to_le_bytes());
        }
        chunk(11, 0, &b)
    }

    fn frame(verts: &[[f32; 3]], texs: &[[f32; 2]]) -> Vec<u8> {
        let mut v = (verts.len() as u32).to_le_bytes().to_vec();
        for p in verts {
            for c in p {
                v.extend_from_slice(&c.to_le_bytes());
            }
        }
        let mut t = (texs.len() as u32).to_le_bytes().to_vec();
        for p in texs {
            for c in p {
                t.extend_from_slice(&c.to_le_bytes());
            }
        }
        let mut body = chunk(2, 0, &v);
        body.extend(chunk(4, 0, &t));
        chunk(12, 2, &body)
    }

    fn file(parts: &[Vec<u8>]) -> Vec<u8> {
        chunk(0, parts.len() as u16, &parts.concat())
    }

    const TRI: [[f32; 3]; 3] = [[0.0, 0.0, 0.5], [1.0, 0.0, 0.5], [0.0, 2.0, 0.5]];
    const UV: [[f32; 2]; 3] = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];

    #[test]
    fn a_two_frame_triangle_reads_and_the_section_keeps_three_x_and_y() {
        let moved = TRI.map(|p| [p[0] + 1.0, p[1], p[2]]);
        let b = file(&[header(2, &[0, 1, 2]), frame(&TRI, &UV), frame(&moved, &UV)]);
        let m = decode(&b).unwrap();
        assert_eq!((m.frames.len(), m.verts(), m.faces.len()), (2, 3, 3));
        assert_eq!(m.section_verts(1)[2], [3.0, 6.0]);
        assert!(m.texcoords_static());
    }

    #[test]
    fn a_frame_count_the_header_does_not_declare_is_refused() {
        let b = file(&[header(3, &[0, 1, 2]), frame(&TRI, &UV), frame(&TRI, &UV)]);
        assert_eq!(
            decode(&b).unwrap_err().reason,
            Reason::FrameCount {
                declared: 3,
                found: 2
            }
        );
    }

    #[test]
    fn a_face_past_the_vertices_and_a_ragged_frame_are_refused() {
        let b = file(&[header(1, &[0, 1, 3]), frame(&TRI, &UV)]);
        assert_eq!(
            decode(&b).unwrap_err().reason,
            Reason::IndexOutOfRange { index: 3, verts: 3 }
        );
        let two = &TRI[..2];
        let b = file(&[
            header(2, &[0, 1, 0]),
            frame(&TRI, &UV),
            frame(two, &UV[..2]),
        ]);
        assert!(matches!(
            decode(&b).unwrap_err().reason,
            Reason::CountMismatch {
                id: 2,
                count: 2,
                verts: 3
            }
        ));
        let b = file(&[header(1, &[0, 1, 2]), frame(&TRI, &UV[..2])]);
        assert!(matches!(
            decode(&b).unwrap_err().reason,
            Reason::CountMismatch { id: 4, .. }
        ));
    }
}
