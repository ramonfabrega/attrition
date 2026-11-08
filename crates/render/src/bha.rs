//! The animation file, `.BHa` (`docs/FORMATS.md` §3, "The animation
//! file").
//!
//! Written from reading `AnimMgr::force_load` and `AnimObj::load_hier` in
//! the decompile, with the PDB's `ChunkHeader`, `AnimObj` and
//! `AnimKeyFrame` for the names. `rondata::artdata::key_times` reads the
//! root's durations for the simulation, and is diff-backed; this reads the
//! whole file, and the two are held to agree on every file the install
//! ships (`rondata::artdata`'s survey).
//!
//! ```text
//! id 0  file                       num_chunks = roots + tracks
//!   id 8  node                     num_chunks = 1 + children
//!     id 7  keys        u32 n, then room for whole 36-byte keys:
//!                       f32 seconds, f32 quat x, y, z, w,
//!                       f32 trans x, y, z, u32 (never read)
//!     id 8  child node …
//!   id 10 track         u32 n, n × 24 bytes      (skipped by force_load)
//! ```
//!
//! **`force_load` loads the first root and skips the rest**: a second
//! `id 8` finds its `AnimObj` already made and is closed unread, and
//! `id 10` is closed unread always. Any other id there is never closed,
//! and the loop gives up after a thousand turns with "Invalid Animation
//! File" — so this reader refuses one too.
//!
//! **The key count is the chunk's own field, and the keys need not fill
//! the chunk**: `load_hier` reads that many and never compares, and the
//! next node starts where the key chunk's `size` says. A chunk with room
//! for one more key than its count is common; one whose count overruns
//! its room is not one this reader understands.

use crate::bh3::{hundredths, thousandths};
use crate::chunk::{self, Chunk, DecodeError, Reason, err, f32_at, u32_at};

/// `AnimObj::load_hier`'s bound on a node's key count: `local_18 < 0x186a1`.
pub const MAX_KEYS: u32 = 100_000;

/// A decoded animation.
#[derive(Clone, Debug, PartialEq)]
pub struct Anim {
    /// Every node of every root, depth first, parents before children.
    pub nodes: Vec<AnimNode>,
    /// How many root (`id 8`) chunks the file holds; only the first plays.
    pub roots: usize,
    /// Each `id 10` chunk's element count.
    pub tracks: Vec<u32>,
}

/// One node's keys: an `AnimObj`.
#[derive(Clone, Debug, PartialEq)]
pub struct AnimNode {
    pub parent: Option<usize>,
    pub keys: Vec<Key>,
    /// Whole keys of room the chunk has past its count.
    pub spare: usize,
}

/// One 36-byte key, as stored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Key {
    /// The key's own duration in seconds.
    pub seconds: f32,
    /// `AnimKeyFrame::q`.
    pub rotation: [f32; 4],
    /// `AnimKeyFrame::t`.
    pub translation: [f32; 3],
    /// The ninth word, which `load_hier` steps over.
    pub unread: u32,
}

impl Key {
    /// The rotation `load_hier` keeps: truncated to thousandths.
    pub fn loaded_rotation(&self) -> [f32; 4] {
        self.rotation.map(thousandths)
    }

    /// The translation `load_hier` keeps: truncated to hundredths.
    pub fn loaded_translation(&self) -> [f32; 3] {
        self.translation.map(hundredths)
    }
}

impl AnimNode {
    /// `AnimObj::key_times`: each key's `int(seconds · 1000)`, masked to
    /// sixteen bits, accumulated into a `u16`.
    pub fn times(&self) -> Vec<u16> {
        let mut acc: i64 = 0;
        self.keys
            .iter()
            .map(|k| {
                acc += i64::from((k.seconds * 1000.0) as i32 & 0xffff);
                (acc & 0xffff) as u16
            })
            .collect()
    }
}

impl Anim {
    /// The nodes of the first root — the one `force_load` plays.
    pub fn played(&self) -> &[AnimNode] {
        let end = self.nodes[1..]
            .iter()
            .position(|n| n.parent.is_none())
            .map_or(self.nodes.len(), |i| i + 1);
        &self.nodes[..end]
    }

    /// The number of children of each played node, in file order — the
    /// shape a model's hierarchy shares (`crate::bh3::Model::shape`).
    pub fn shape(&self) -> Vec<usize> {
        let nodes = self.played();
        let mut kids = vec![0; nodes.len()];
        for n in nodes {
            if let Some(p) = n.parent {
                kids[p] += 1;
            }
        }
        kids
    }
}

/// Decode a `.BHa` file's bytes.
pub fn decode(bytes: &[u8]) -> Result<Anim, DecodeError> {
    let file = chunk::file(bytes)?;
    let mut nodes = Vec::new();
    let mut roots = 0;
    let mut tracks = Vec::new();
    for top in chunk::children(bytes, &file, file.body(), 0)? {
        match top.id {
            8 => {
                node(bytes, &top, None, &mut nodes)?;
                roots += 1;
            }
            10 => {
                let n = u32_at(bytes, top.body())?;
                let size = (top.end - top.at) as u32;
                if 12 + n as usize * 24 != top.end - top.at {
                    return err(
                        top.at,
                        Reason::ArraySize {
                            id: 10,
                            count: n,
                            stride: 24,
                            size,
                        },
                    );
                }
                tracks.push(n);
            }
            id => return err(top.at, Reason::UnknownId { id, parent: 0 }),
        }
    }
    if roots == 0 {
        return err(file.at, Reason::Missing { id: 8 });
    }
    Ok(Anim {
        nodes,
        roots,
        tracks,
    })
}

fn node(
    bytes: &[u8],
    wrap: &Chunk,
    parent: Option<usize>,
    out: &mut Vec<AnimNode>,
) -> Result<(), DecodeError> {
    let data = chunk::header(bytes, wrap.body(), wrap.end)?;
    if data.id != 7 {
        return err(data.at, Reason::NodeShape { first: data.id });
    }
    let n = u32_at(bytes, data.body())?;
    let room = data.end - data.at - 12;
    if n > MAX_KEYS {
        return err(data.at, Reason::TooManyKeys { keys: n });
    }
    if room % 36 != 0 || n as usize * 36 > room {
        return err(data.at, Reason::KeyRoom { keys: n, room });
    }
    let at = data.body() + 4;
    let keys = (0..n as usize)
        .map(|i| {
            let p = at + 36 * i;
            let f = |k: usize| f32_at(bytes, p + 4 * k);
            Ok(Key {
                seconds: f(0)?,
                rotation: [f(1)?, f(2)?, f(3)?, f(4)?],
                translation: [f(5)?, f(6)?, f(7)?],
                unread: u32_at(bytes, p + 32)?,
            })
        })
        .collect::<Result<Vec<_>, DecodeError>>()?;
    let me = out.len();
    out.push(AnimNode {
        parent,
        keys,
        spare: room / 36 - n as usize,
    });
    for c in chunk::children(bytes, wrap, data.end, 1)? {
        if c.id != 8 {
            return err(
                c.at,
                Reason::UnknownId {
                    id: c.id,
                    parent: 8,
                },
            );
        }
        node(bytes, &c, Some(me), out)?;
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

    fn keys(declared: u32, secs: &[f32]) -> Vec<u8> {
        let mut b = declared.to_le_bytes().to_vec();
        for (i, s) in secs.iter().enumerate() {
            for f in [*s, 0.0, 0.0, 0.0, 1.0, i as f32, 0.5, -0.25] {
                b.extend_from_slice(&f.to_le_bytes());
            }
            b.extend_from_slice(&0xdead_beefu32.to_le_bytes());
        }
        chunk(7, 0, &b)
    }

    fn node(k: Vec<u8>, kids: &[Vec<u8>]) -> Vec<u8> {
        let mut body = k;
        for c in kids {
            body.extend_from_slice(c);
        }
        chunk(8, 1 + kids.len() as u16, &body)
    }

    /// Hand-built: a root of three keys (room for four), one child, and a
    /// skipped track.
    fn sample(declared: u32) -> Vec<u8> {
        let child = node(keys(2, &[0.1, 0.2]), &[]);
        let root = node(keys(declared, &[0.5, 0.25, 0.75, 0.0]), &[child]);
        let mut track = 2u32.to_le_bytes().to_vec();
        track.extend_from_slice(&[0u8; 48]);
        let mut body = root;
        body.extend(chunk(10, 0, &track));
        chunk(0, 2, &body)
    }

    #[test]
    fn a_hand_built_animation_decodes_whole() {
        let a = decode(&sample(3)).unwrap();
        assert_eq!((a.roots, a.tracks.clone()), (1, vec![2]));
        assert_eq!(a.nodes.len(), 2);
        assert_eq!((a.nodes[0].keys.len(), a.nodes[0].spare), (3, 1));
        assert_eq!(a.nodes[0].times(), [500, 750, 1500]);
        assert_eq!(a.nodes[0].keys[2].translation, [2.0, 0.5, -0.25]);
        assert_eq!(a.nodes[0].keys[2].unread, 0xdead_beef);
        assert_eq!(a.shape(), [1, 0]);
    }

    #[test]
    fn a_count_past_the_room_is_named() {
        let e = decode(&sample(5)).unwrap_err();
        assert_eq!(e.reason, Reason::KeyRoom { keys: 5, room: 144 });
    }
}
