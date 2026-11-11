//! The model file, `.BH3` (`docs/FORMATS.md` §3, "The model file").
//!
//! Written from reading `BH3Method::load`, `load_bh3`, `load_vertex_data`,
//! `load_verts`, `load_faces`, `load_texs`, `load_hier` and
//! `BH3LightMethod::load_norms` in the decompile, with the PDB's
//! `ChunkHeader`, `HierObj`, `Quat<float>` and `Transform<float>` for the
//! names. Never transcribed: the original fills render packets, and this
//! returns plain arrays.
//!
//! ```text
//! id 0  file                       num_chunks = 2
//!   id 1  vertex data              num_chunks = 4
//!     id 2  vertices    u32 n, n × {f32 x, y, z, w}   (w = 1)
//!     id 3  normals     u32 n, n × {f32 x, y, z}, then 4n bytes unread
//!     id 4  texcoords   u32 n, n × {f32 u, v}
//!     id 5  faces       u32 n, n × u16               (n / 3 triangles)
//!   id 6  node                     num_chunks = 1 + children
//!     id 7  node data   u32 start, u32 num, u32 name_len, name,
//!                       f32 quat x, y, z, w, f32 trans x, y, z, w
//!     id 6  child node …
//! ```
//!
//! **The decoder knows when it is wrong.** Every chunk ends inside its
//! parent and every container's `num_chunks` is its child count; each array
//! chunk is exactly `12 + n · stride`; the normals and texcoords are one per
//! vertex; the faces are whole triangles whose indices are vertices; a
//! node's data is exactly its name and transform, and its vertex range lies
//! inside the vertex array; every vertex, texcoord and transform value is
//! finite and every vertex's `w` is 1. The normals alone are kept as
//! stored and unchecked, because nothing a unit loads with reads them;
//! they are the chunk's first `12n` bytes, as `BH3LightMethod::load_norms`
//! copies them (`n · 12` into an `Array<Vector<float>>`), and the last
//! `4n` are the writer's filler (`0xcdcdcdcd` in 585 files; item 4122). The original checks none of this — it `memcpy`s — so a file
//! that passes here is one whose every byte has a stated meaning.

use crate::chunk::{self, Chunk, DecodeError, Reason, err, f32_at, u32_at};

/// A decoded model.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    /// `id 2`: `Point<float>` each, `w` always 1. `load_verts` copies the
    /// first three components of each into the render packet.
    pub verts: Vec<[f32; 3]>,
    /// `id 3`: one `Vector<float>` per vertex, packed twelve bytes apart in
    /// the chunk's first `12n` bytes, as `BH3LightMethod::load_norms@
    /// 00914b40` copies them (`n · 12`), as stored and unchecked —
    /// `BH3Method::load_vertex_data` never reads this chunk. Item 4122:
    /// read at the array's own sixteen-byte stride (as until then), only
    /// a third of a piece's normals were unit length and they sat at a
    /// median 90° to the faces; read packed, all 401 of the medium
    /// tank's are unit (`docs/FORMATS.md` §3).
    pub normals: Vec<[f32; 3]>,
    /// `id 4`: one `{u, v}` per vertex.
    pub texcoords: Vec<[f32; 2]>,
    /// `id 5`: the index list, three to a triangle.
    pub faces: Vec<[u16; 3]>,
    /// `id 6`/`id 7`: the node tree in file order (depth first, parents
    /// before children).
    pub nodes: Vec<Node>,
}

/// One node of the hierarchy, a `HierObj` as `load_hier` fills it.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub name: String,
    /// Index of the parent in [`Model::nodes`]; `None` for a root.
    pub parent: Option<usize>,
    /// `HierObj::start` and `num`: the run of vertices this node carries.
    pub start: u32,
    pub num: u32,
    /// As stored. `load_hier` truncates them on load —
    /// [`Node::loaded_rotation`], [`Node::loaded_translation`].
    pub rotation: [f32; 4],
    pub translation: [f32; 3],
    /// The eighth float, read into the point's `w`. It equals the
    /// rotation's `x` in every file the install ships.
    pub trans_w: f32,
}

/// `(int)(v · 1000) · 0.001` and `(int)(v · 100) · 0.01`: the precision
/// `load_hier` (and `AnimObj::load_hier`, for a key) keeps.
pub(crate) fn thousandths(v: f32) -> f32 {
    ((v * 1000.0) as i32) as f32 * 0.001
}

pub(crate) fn hundredths(v: f32) -> f32 {
    ((v * 100.0) as i32) as f32 * 0.01
}

impl Node {
    /// The rotation `load_hier` keeps: each component truncated to
    /// thousandths.
    pub fn loaded_rotation(&self) -> [f32; 4] {
        self.rotation.map(thousandths)
    }

    /// The translation `load_hier` keeps: each component truncated to
    /// hundredths.
    pub fn loaded_translation(&self) -> [f32; 3] {
        self.translation.map(hundredths)
    }
}

/// The axis-aligned box the vertices span.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl Bounds {
    pub fn contains(&self, p: [f32; 3]) -> bool {
        (0..3).all(|k| self.min[k] <= p[k] && p[k] <= self.max[k])
    }
}

impl Model {
    /// The box the vertices span; `None` for a model with none. The file
    /// stores no bounds of its own — this is derived.
    pub fn bounds(&self) -> Option<Bounds> {
        let first = *self.verts.first()?;
        let mut b = Bounds {
            min: first,
            max: first,
        };
        for v in &self.verts {
            for (k, &c) in v.iter().enumerate() {
                b.min[k] = b.min[k].min(c);
                b.max[k] = b.max[k].max(c);
            }
        }
        Some(b)
    }

    /// The number of children of each node, in file order — the tree's
    /// shape, which an animation must share (`crate::bha::Anim::shape`).
    pub fn shape(&self) -> Vec<usize> {
        let mut kids = vec![0; self.nodes.len()];
        for n in &self.nodes {
            if let Some(p) = n.parent {
                kids[p] += 1;
            }
        }
        kids
    }
}

/// An array chunk's count and the offset its elements start at, held to
/// `12 + count · stride == size`.
fn array(bytes: &[u8], c: &Chunk, stride: usize) -> Result<(usize, usize), DecodeError> {
    let count = u32_at(bytes, c.body())?;
    let size = (c.end - c.at) as u32;
    if 12 + count as usize * stride != c.end - c.at {
        return err(
            c.at,
            Reason::ArraySize {
                id: c.id,
                count,
                stride,
                size,
            },
        );
    }
    Ok((count as usize, c.body() + 4))
}

/// Decode a `.BH3` file's bytes.
pub fn decode(bytes: &[u8]) -> Result<Model, DecodeError> {
    let file = chunk::file(bytes)?;
    let mut verts = None;
    let mut normals = Vec::new();
    let mut texcoords = Vec::new();
    let mut faces_raw: Option<(usize, usize, usize)> = None;
    let mut nodes = Vec::new();
    let mut have_hier = false;
    for top in chunk::children(bytes, &file, file.body(), 0)? {
        match top.id {
            1 => {
                for c in chunk::children(bytes, &top, top.body(), 0)? {
                    match c.id {
                        2 => {
                            let (n, at) = array(bytes, &c, 16)?;
                            let mut v = Vec::with_capacity(n);
                            for i in 0..n {
                                let p = at + 16 * i;
                                let w = u32_at(bytes, p + 12)?;
                                if w != 1.0f32.to_bits() {
                                    return err(p, Reason::VertexW { w_bits: w });
                                }
                                v.push([
                                    f32_at(bytes, p)?,
                                    f32_at(bytes, p + 4)?,
                                    f32_at(bytes, p + 8)?,
                                ]);
                            }
                            verts = Some(v);
                        }
                        3 => {
                            let (n, at) = array(bytes, &c, 16)?;
                            // Raw, packed: `load_norms` copies `n · 12`
                            // bytes; the chunk's last `4n` are filler,
                            // never read. The method a unit loads with
                            // never reads this chunk.
                            normals = (0..n)
                                .map(|i| {
                                    let p = at + 12 * i;
                                    Ok([0, 4, 8].map(|k| {
                                        let b = &bytes[p + k..p + k + 4];
                                        f32::from_bits(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                                    }))
                                })
                                .collect::<Result<_, DecodeError>>()?;
                        }
                        4 => {
                            let (n, at) = array(bytes, &c, 8)?;
                            texcoords = (0..n)
                                .map(|i| {
                                    Ok([f32_at(bytes, at + 8 * i)?, f32_at(bytes, at + 8 * i + 4)?])
                                })
                                .collect::<Result<_, DecodeError>>()?;
                        }
                        5 => {
                            let (n, at) = array(bytes, &c, 2)?;
                            faces_raw = Some((c.at, n, at));
                        }
                        id => return err(c.at, Reason::UnknownId { id, parent: 1 }),
                    }
                }
            }
            6 => {
                hier(bytes, &top, None, &mut nodes)?;
                have_hier = true;
            }
            id => return err(top.at, Reason::UnknownId { id, parent: 0 }),
        }
    }
    let Some(verts) = verts else {
        return err(file.at, Reason::Missing { id: 2 });
    };
    if !have_hier {
        return err(file.at, Reason::Missing { id: 6 });
    }
    let nv = verts.len();
    for (id, count) in [(3u16, normals.len()), (4, texcoords.len())] {
        if count != nv {
            return err(
                file.at,
                Reason::CountMismatch {
                    id,
                    count,
                    verts: nv,
                },
            );
        }
    }
    let Some((faces_at, n, at)) = faces_raw else {
        return err(file.at, Reason::Missing { id: 5 });
    };
    if n % 3 != 0 {
        return err(faces_at, Reason::PartialTriangle { indices: n });
    }
    let mut faces = Vec::with_capacity(n / 3);
    for t in 0..n / 3 {
        let mut tri = [0u16; 3];
        for (k, slot) in tri.iter_mut().enumerate() {
            let p = at + 2 * (3 * t + k);
            let i = u16::from_le_bytes([bytes[p], bytes[p + 1]]);
            if usize::from(i) >= nv {
                return err(
                    p,
                    Reason::IndexOutOfRange {
                        index: i,
                        verts: nv,
                    },
                );
            }
            *slot = i;
        }
        faces.push(tri);
    }
    for node in &nodes {
        if node.start as usize + node.num as usize > nv {
            return err(
                file.at,
                Reason::RangePastVerts {
                    start: node.start,
                    num: node.num,
                    verts: nv,
                },
            );
        }
    }
    Ok(Model {
        verts,
        normals,
        texcoords,
        faces,
        nodes,
    })
}

/// One `id 6` node and its subtree, appended depth first.
fn hier(
    bytes: &[u8],
    wrap: &Chunk,
    parent: Option<usize>,
    out: &mut Vec<Node>,
) -> Result<(), DecodeError> {
    let data = chunk::header(bytes, wrap.body(), wrap.end)?;
    if data.id != 7 {
        return err(data.at, Reason::NodeShape { first: data.id });
    }
    let start = u32_at(bytes, data.body())?;
    let num = u32_at(bytes, data.body() + 4)?;
    let name_len = u32_at(bytes, data.body() + 8)?;
    let name_at = data.body() + 12;
    let size = (data.end - data.at) as u32;
    if 20 + name_len as usize + 32 != data.end - data.at {
        return err(data.at, Reason::NodeSize { name_len, size });
    }
    let raw = &bytes[name_at..name_at + name_len as usize];
    let name = String::from_utf8_lossy(raw.split(|&b| b == 0).next().unwrap_or(&[])).into_owned();
    let tf = name_at + name_len as usize;
    let f = |k: usize| f32_at(bytes, tf + 4 * k);
    let me = out.len();
    out.push(Node {
        name,
        parent,
        start,
        num,
        rotation: [f(0)?, f(1)?, f(2)?, f(3)?],
        translation: [f(4)?, f(5)?, f(6)?],
        trans_w: f32::from_bits(u32_at(bytes, tf + 28)?),
    });
    for c in chunk::children(bytes, wrap, data.end, 1)? {
        if c.id != 6 {
            return err(
                c.at,
                Reason::UnknownId {
                    id: c.id,
                    parent: 6,
                },
            );
        }
        hier(bytes, &c, Some(me), out)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(id: u16, n: u16, body: &[u8]) -> Vec<u8> {
        let mut v = (8 + body.len() as u32).to_le_bytes().to_vec();
        v.extend_from_slice(&id.to_le_bytes());
        v.extend_from_slice(&n.to_le_bytes());
        v.extend_from_slice(body);
        v
    }

    fn array(id: u16, n: u32, elems: &[u8]) -> Vec<u8> {
        let mut b = n.to_le_bytes().to_vec();
        b.extend_from_slice(elems);
        chunk(id, 0, &b)
    }

    fn floats(v: &[f32]) -> Vec<u8> {
        v.iter().flat_map(|f| f.to_le_bytes()).collect()
    }

    fn node(name: &str, start: u32, num: u32, kids: &[Vec<u8>]) -> Vec<u8> {
        let mut d = start.to_le_bytes().to_vec();
        d.extend_from_slice(&num.to_le_bytes());
        let mut n = name.as_bytes().to_vec();
        n.push(0);
        d.extend_from_slice(&(n.len() as u32).to_le_bytes());
        d.extend_from_slice(&n);
        d.extend_from_slice(&floats(&[0.0, 0.0, 0.0, 1.0, 1.5, 2.25, 3.0, 0.0]));
        let mut body = chunk(7, 0, &d);
        for k in kids {
            body.extend_from_slice(k);
        }
        chunk(6, 1 + kids.len() as u16, &body)
    }

    /// A hand-built triangle — no byte of it comes from the install.
    fn triangle(index2: u16) -> Vec<u8> {
        let v = floats(&[0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 2.0, -1.0, 1.0]);
        let nrm = floats(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        let tex = floats(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
        let idx: Vec<u8> = [0u16, 1, index2]
            .iter()
            .flat_map(|i| i.to_le_bytes())
            .collect();
        let mut vd = array(2, 3, &v);
        vd.extend(array(3, 3, &nrm));
        vd.extend(array(4, 3, &tex));
        vd.extend(array(5, 3, &idx));
        let mut body = chunk(1, 4, &vd);
        body.extend(node("root", 0, 1, &[node("tip", 1, 2, &[])]));
        chunk(0, 2, &body)
    }

    #[test]
    fn a_hand_built_model_decodes_whole() {
        let m = decode(&triangle(2)).unwrap();
        assert_eq!(
            m.verts,
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 2.0, -1.0]]
        );
        assert_eq!(m.faces, [[0, 1, 2]]);
        assert_eq!(m.texcoords[2], [0.0, 1.0]);
        assert_eq!(m.nodes.len(), 2);
        assert_eq!(
            (m.nodes[1].name.as_str(), m.nodes[1].parent),
            ("tip", Some(0))
        );
        assert_eq!((m.nodes[1].start, m.nodes[1].num), (1, 2));
        assert_eq!(m.nodes[0].translation, [1.5, 2.25, 3.0]);
        assert_eq!(m.shape(), [1, 0]);
        let b = m.bounds().unwrap();
        assert_eq!((b.min, b.max), ([0.0, 0.0, -1.0], [1.0, 2.0, 0.0]));
        assert!(m.verts.iter().all(|&v| b.contains(v)));
    }

    #[test]
    fn a_face_index_past_the_vertices_is_named() {
        let e = decode(&triangle(3)).unwrap_err();
        assert_eq!(e.reason, Reason::IndexOutOfRange { index: 3, verts: 3 });
    }

    #[test]
    fn a_vertex_whose_w_is_not_one_is_named() {
        let mut b = triangle(2);
        // The first vertex's w: file(8) + vertex data(8) + id 2 header(8) + count(4) + xyz(12).
        let at = 8 + 8 + 8 + 4 + 12;
        b[at..at + 4].copy_from_slice(&0.5f32.to_le_bytes());
        assert!(matches!(
            decode(&b).unwrap_err().reason,
            Reason::VertexW { .. }
        ));
    }

    #[test]
    fn the_loader_s_truncation_is_toward_zero() {
        assert_eq!(thousandths(0.70710677), 0.707);
        assert_eq!(thousandths(-0.70710677), -0.707);
        assert_eq!(hundredths(16.334035), 16.33);
        assert_eq!(hundredths(-4.6498594), -4.64);
    }
}
