//! The chunk stream both model formats are written in (`docs/FORMATS.md`
//! §3, "The chunk container").
//!
//! A chunk is an eight-byte header, the PDB's `ChunkHeader` — `{ulong size;
//! ushort id; ushort num_chunks}` — and a body. `size` is measured from the
//! `size` field itself, so a chunk at `at` ends at `at + size`; that is
//! `ChunkRead`'s own `(int)&header->size + header->size`. A container's
//! `num_chunks` is the count of the chunks directly inside it, and a leaf's
//! is zero: the whole of the install agrees, and the readers here hold every
//! file to it, which is what makes the walk self-checking.

use std::fmt;

/// One chunk: where its header sits, what it says, and where it ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub at: usize,
    pub id: u16,
    pub num_chunks: u16,
    pub end: usize,
}

impl Chunk {
    /// The first byte after the header.
    pub fn body(&self) -> usize {
        self.at + 8
    }
}

/// Why a file is not one these readers understand, and where.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodeError {
    pub at: usize,
    pub reason: Reason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    /// A header that does not fit before its parent ends.
    HeaderPastEnd,
    /// A chunk whose `size` is under eight or runs past its parent.
    Overrun { size: u32, parent_end: usize },
    /// The outermost chunk is not id 0, or does not end where the file does.
    NotAFile { id: u16, size: u32, len: usize },
    /// A container whose `num_chunks` is not the count of chunks inside it.
    ChildCount { declared: u16, found: usize },
    /// A chunk id the reader does not know in this place.
    UnknownId { id: u16, parent: u16 },
    /// An array chunk whose `12 + count · stride` is not its size.
    ArraySize {
        id: u16,
        count: u32,
        stride: usize,
        size: u32,
    },
    /// An array whose count disagrees with the vertex count.
    CountMismatch { id: u16, count: usize, verts: usize },
    /// A face list that is not whole triangles.
    PartialTriangle { indices: usize },
    /// A face index at or past the vertex count.
    IndexOutOfRange { index: u16, verts: usize },
    /// A node whose vertex range runs past the vertex array.
    RangePastVerts { start: u32, num: u32, verts: usize },
    /// A node's data chunk whose name and transform do not fill it.
    NodeSize { name_len: u32, size: u32 },
    /// A node wrapper whose first chunk is not its data.
    NodeShape { first: u16 },
    /// A key chunk whose keys overrun it, or whose room is not whole keys.
    KeyRoom { keys: u32, room: usize },
    /// More keys than `AnimObj::load_hier` accepts (100,000).
    TooManyKeys { keys: u32 },
    /// A value that is not finite where a coordinate must be.
    NotFinite,
    /// A vertex whose fourth component is not 1.
    VertexW { w_bits: u32 },
    /// A file with no geometry or no hierarchy.
    Missing { id: u16 },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "at byte {}: {:?}", self.at, self.reason)
    }
}

impl std::error::Error for DecodeError {}

pub(crate) fn err<T>(at: usize, reason: Reason) -> Result<T, DecodeError> {
    Err(DecodeError { at, reason })
}

pub(crate) fn u32_at(bytes: &[u8], at: usize) -> Result<u32, DecodeError> {
    bytes
        .get(at..at + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .ok_or(DecodeError {
            at,
            reason: Reason::HeaderPastEnd,
        })
}

pub(crate) fn f32_at(bytes: &[u8], at: usize) -> Result<f32, DecodeError> {
    let v = f32::from_bits(u32_at(bytes, at)?);
    if v.is_finite() {
        Ok(v)
    } else {
        err(at, Reason::NotFinite)
    }
}

/// The header at `at`, which must end by `parent_end`.
pub fn header(bytes: &[u8], at: usize, parent_end: usize) -> Result<Chunk, DecodeError> {
    if at + 8 > parent_end || parent_end > bytes.len() {
        return err(at, Reason::HeaderPastEnd);
    }
    let size = u32_at(bytes, at)?;
    let id = u16::from_le_bytes([bytes[at + 4], bytes[at + 5]]);
    let num_chunks = u16::from_le_bytes([bytes[at + 6], bytes[at + 7]]);
    let end = at.saturating_add(size as usize);
    if size < 8 || end > parent_end {
        return err(at, Reason::Overrun { size, parent_end });
    }
    Ok(Chunk {
        at,
        id,
        num_chunks,
        end,
    })
}

/// The chunks laid end to end from `from` to `parent.end`, held to the
/// parent's `num_chunks` — with `already` chunks before `from` that the
/// caller read by hand (a node's own data chunk) counted in.
pub fn children(
    bytes: &[u8],
    parent: &Chunk,
    from: usize,
    already: usize,
) -> Result<Vec<Chunk>, DecodeError> {
    let mut out = Vec::new();
    let mut at = from;
    while at < parent.end {
        let c = header(bytes, at, parent.end)?;
        at = c.end;
        out.push(c);
    }
    if out.len() + already != usize::from(parent.num_chunks) {
        return err(
            parent.at,
            Reason::ChildCount {
                declared: parent.num_chunks,
                found: out.len() + already,
            },
        );
    }
    Ok(out)
}

/// The outermost chunk: id 0, ending exactly where the file does.
pub fn file(bytes: &[u8]) -> Result<Chunk, DecodeError> {
    let c = header(bytes, 0, bytes.len())?;
    if c.id != 0 || c.end != bytes.len() {
        return err(
            0,
            Reason::NotAFile {
                id: c.id,
                size: (c.end - c.at) as u32,
                len: bytes.len(),
            },
        );
    }
    Ok(c)
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

    #[test]
    fn a_size_counts_from_its_own_field() {
        let inner = chunk(2, 0, &[1, 2, 3, 4]);
        let f = chunk(0, 1, &inner);
        let top = file(&f).unwrap();
        assert_eq!((top.id, top.end), (0, 20));
        let kids = children(&f, &top, top.body(), 0).unwrap();
        assert_eq!(
            kids,
            [Chunk {
                at: 8,
                id: 2,
                num_chunks: 0,
                end: 20
            }]
        );
    }

    #[test]
    fn a_wrong_child_count_or_an_overrun_is_named() {
        let inner = chunk(2, 0, &[]);
        let f = chunk(0, 2, &inner);
        let top = file(&f).unwrap();
        assert_eq!(
            children(&f, &top, 8, 0).unwrap_err().reason,
            Reason::ChildCount {
                declared: 2,
                found: 1
            }
        );
        let mut g = chunk(0, 1, &chunk(2, 0, &[]));
        g[8] = 9; // the child's size now runs a byte past its parent
        let top = file(&g).unwrap();
        assert!(matches!(
            children(&g, &top, 8, 0).unwrap_err().reason,
            Reason::Overrun { size: 9, .. }
        ));
    }
}
