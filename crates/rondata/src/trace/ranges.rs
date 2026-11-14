//! Step R's record (item 4248, `docs/CHECKSUM.md` §25): every range the
//! original's checksum fold took on the open lines, on every frame of one
//! playback, from a `rontrace.ranges` written by a `RON_CHECKSUMS
//! RON_RANGES` build of `tools/trace/` (`tools/trace/ranges.h` has the hook
//! and the format; `tools/trace/ranges.py` is the same reader in Python).
//!
//! Per due `check_all` set — the same sets the checksum hook writes to
//! `rontrace.log` ([`super::checksums`]) — the set's sixteen words and, for
//! each line in the record's mask, every range `CheckSum::walk_function`
//! (and `check_groups`' zlib `adler32`) took, in fold order, with its call
//! site and its bytes as folded. A set is the original's memory on one
//! frame boundary, line by line, without a packet.
//!
//! The record is gigabytes and never enters the repo; it is opened by path
//! and read set by set.

use std::io::{Read, Seek, SeekFrom};

const MAGIC: u32 = 0x524e_4752;
const SET_MAGIC: u32 = 0x5445_5352;
const VERSION: u32 = 1;
/// A set whose ranges did not fit the hook's buffer carries none.
pub const OVER: u32 = 1;
const SET_WORDS: usize = 24;
const ENTRY_BYTES: usize = 16;

/// One due set's header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RangeSet {
    /// The tracer's frame, as a checksum set's ([`super::checksums::Set`]).
    pub frame: i64,
    /// The set's place among the frame's `check_all` calls.
    pub ordinal: u32,
    /// `check_all`'s return into `process_turn`, at [`super::IMAGE_BASE`].
    pub caller: u32,
    /// The frame's `check_all` calls up to and including this one.
    pub calls: u32,
    /// [`OVER`] when its ranges did not fit.
    pub flags: u32,
    /// The fifteen lines, in [`crate::checksum::CHECKSUM_LINES`]' order.
    pub lines: [u32; 15],
    /// `check_all`'s wrapping sum of the fifteen.
    pub total: u32,
    offset: u64,
    nranges: u32,
    nbytes: u32,
}

/// One range the fold took.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Range {
    /// `check_all`'s line index, 0 = units.
    pub line: u8,
    /// 0 = `CheckSum::walk_function`, 1 = `check_groups`' zlib `adler32`.
    pub via: u8,
    /// The fold's return address, at [`super::IMAGE_BASE`].
    pub site: u32,
    /// The range's address in the original's process.
    pub start: u32,
    pub bytes: Vec<u8>,
}

/// An open record: its header and every set's, indexed by one pass over
/// the headers.
pub struct Record {
    file: std::fs::File,
    base: u32,
    /// The lines whose ranges the record keeps, a bit each.
    pub mask: u32,
    pub sets: Vec<RangeSet>,
    /// The record ends inside a set (a take killed mid-write).
    pub truncated: bool,
}

fn words<const N: usize>(b: &[u8]) -> [u32; N] {
    std::array::from_fn(|i| u32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap()))
}

impl Record {
    /// Opens a record and reads every set's header.
    pub fn open(path: &std::path::Path) -> std::io::Result<Record> {
        let mut file = std::fs::File::open(path)?;
        let size = file.metadata()?.len();
        let mut head = [0u8; 32];
        file.read_exact(&mut head)?;
        let h: [u32; 8] = words(&head);
        if h[0] != MAGIC || h[1] != VERSION {
            return Err(std::io::Error::other(format!(
                "{}: not a step R record (magic {:#x}, version {})",
                path.display(),
                h[0],
                h[1]
            )));
        }
        let (base, mask) = (h[2], h[3]);
        let mut sets = Vec::new();
        let mut at = 32u64;
        let mut truncated = false;
        let mut raw = [0u8; SET_WORDS * 4];
        while at + raw.len() as u64 <= size {
            file.seek(SeekFrom::Start(at))?;
            file.read_exact(&mut raw)?;
            let w: [u32; SET_WORDS] = words(&raw);
            if w[0] != SET_MAGIC {
                return Err(std::io::Error::other(format!(
                    "{}: no set at offset {at}",
                    path.display()
                )));
            }
            let (nranges, nbytes) = (w[6], w[7]);
            let len = raw.len() as u64
                + u64::from(nranges) * ENTRY_BYTES as u64
                + u64::from(nbytes)
                + u64::from((4 - (nbytes & 3)) & 3);
            if at + len > size {
                truncated = true;
                break;
            }
            sets.push(RangeSet {
                frame: i64::from(w[1] as i32),
                ordinal: w[2],
                caller: w[3].wrapping_sub(base).wrapping_add(super::IMAGE_BASE),
                calls: w[4],
                flags: w[5],
                lines: std::array::from_fn(|i| w[8 + i]),
                total: w[23],
                offset: at,
                nranges,
                nbytes,
            });
            at += len;
        }
        truncated |= at < size;
        Ok(Record {
            file,
            base,
            mask,
            sets,
            truncated,
        })
    }

    /// Whether the record keeps line `i`'s ranges.
    pub fn keeps(&self, i: usize) -> bool {
        i < 32 && (self.mask >> i) & 1 == 1
    }

    /// The first set of trace frame `frame`, as §24.1 pairs a set with a
    /// packet: logger frame `frame + 1`, our `sim.frame == frame + 1`.
    pub fn first_of(&self, frame: i64) -> Option<&RangeSet> {
        let i = self.sets.partition_point(|s| s.frame < frame);
        self.sets.get(i).filter(|s| s.frame == frame)
    }

    /// A set's ranges, in fold order.
    pub fn ranges(&mut self, set: &RangeSet) -> std::io::Result<Vec<Range>> {
        self.file
            .seek(SeekFrom::Start(set.offset + (SET_WORDS * 4) as u64))?;
        let mut ents = vec![0u8; set.nranges as usize * ENTRY_BYTES];
        self.file.read_exact(&mut ents)?;
        let mut data = vec![0u8; set.nbytes as usize];
        self.file.read_exact(&mut data)?;
        let mut out = Vec::with_capacity(set.nranges as usize);
        let mut k = 0usize;
        for e in ents.as_chunks::<ENTRY_BYTES>().0 {
            let w: [u32; 4] = words(e);
            let n = w[3] as usize;
            let Some(bytes) = data.get(k..k + n) else {
                return Err(std::io::Error::other("a set's ranges run past its bytes"));
            };
            out.push(Range {
                line: e[0],
                via: e[1],
                site: w[1].wrapping_sub(self.base).wrapping_add(super::IMAGE_BASE),
                start: w[2],
                bytes: bytes.to_vec(),
            });
            k += n;
        }
        Ok(out)
    }
}

/// Adler-32 over a line's ranges, as the fold sums them.
pub fn adler(ranges: &[Range], line: u8) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for r in ranges.iter().filter(|r| r.line == line) {
        for &x in &r.bytes {
            a = (a + u32::from(x)) % 65521;
            b = (b + a) % 65521;
        }
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push(out: &mut Vec<u8>, w: &[u32]) {
        for x in w {
            out.extend_from_slice(&x.to_le_bytes());
        }
    }

    /// A record as `ranges.h` writes one: base 0x500000, two sets.
    fn record(cut: usize) -> std::path::PathBuf {
        let mut b = Vec::new();
        push(&mut b, &[MAGIC, VERSION, 0x50_0000, 0xff, 0, 0, 0, 0]);
        for (frame, ord, data) in [(-1i32, 0u32, &b"abc"[..]), (0, 0, &b"wxyz"[..])] {
            let mut lines = [1u32; 16];
            let fold = vec![Range {
                line: 5,
                via: 0,
                site: 0,
                start: 0,
                bytes: data.to_vec(),
            }];
            lines[5] = adler(&fold, 5);
            push(
                &mut b,
                &[
                    SET_MAGIC,
                    frame as u32,
                    ord,
                    0x53_fb68,
                    1,
                    0,
                    1,
                    data.len() as u32,
                ],
            );
            push(&mut b, &lines);
            b.extend_from_slice(&[5, 0, 0, 0]);
            push(&mut b, &[0x80_8419, 0x1234, data.len() as u32]);
            b.extend_from_slice(data);
            b.resize(b.len() + ((4 - (data.len() & 3)) & 3), 0);
        }
        b.truncate(b.len() - cut);
        let p = std::env::temp_dir().join(format!("ranges-{}-{cut}.bin", std::process::id()));
        std::fs::write(&p, &b).unwrap();
        p
    }

    #[test]
    fn a_record_reads_back_set_by_set() {
        let p = record(0);
        let mut r = Record::open(&p).unwrap();
        let _ = std::fs::remove_file(&p);
        assert!(!r.truncated);
        assert_eq!(r.sets.len(), 2);
        assert!(r.keeps(7) && !r.keeps(8));
        let s = r.first_of(0).unwrap().clone();
        assert_eq!((s.frame, s.ordinal, s.caller), (0, 0, 0x43_fb68));
        let got = r.ranges(&s).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(
            (got[0].line, got[0].site, got[0].start),
            (5, 0x70_8419, 0x1234)
        );
        assert_eq!(got[0].bytes, b"wxyz");
        assert_eq!(
            adler(&got, 5),
            s.lines[5],
            "the ranges sum to the set's word"
        );
        assert!(r.first_of(1).is_none());
        let s = r.first_of(-1).unwrap().clone();
        assert_eq!(r.ranges(&s).unwrap()[0].bytes, b"abc");
    }

    #[test]
    fn a_record_cut_inside_a_set_says_so() {
        let p = record(3);
        let r = Record::open(&p).unwrap();
        let _ = std::fs::remove_file(&p);
        assert!(r.truncated);
        assert_eq!(r.sets.len(), 1, "the whole set before the cut is read");
    }
}
