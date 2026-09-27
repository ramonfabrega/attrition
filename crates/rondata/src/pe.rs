//! Just enough of the PE container to read a named global out of
//! `riseofnations.exe`.
//!
//! The symbols say **where** a table is — `rise.pdb`'s `S_GDATA32` records
//! carry a `section:offset` pair — and the section table says what file
//! offset that is. Everything else about the file is somebody else's
//! problem: there is no import walking, no relocation, no disassembly.
//!
//! It exists because a table read out of the executable by hand and typed
//! into a `const` is a reading, and a reading is what this project converts
//! into a check wherever it can. `sim::world::MOVE_289` is the first
//! customer: 289 pairs, one of which is a typo in the shipped data, and
//! nothing but the file itself can say so (`docs/ORDERS.md` §6.8).

/// A mapped image: the sections, and the bytes.
pub struct Pe {
    bytes: Vec<u8>,
    image_base: u32,
    /// `(virtual address, virtual size, raw offset, raw size)` per section.
    sections: Vec<(u32, u32, u32, u32)>,
}

fn u16_at(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(i..i + 2)?.try_into().ok()?))
}

fn u32_at(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(i..i + 4)?.try_into().ok()?))
}

impl Pe {
    /// Reads and indexes a PE file. `None` if it is not one.
    pub fn open(path: &str) -> Option<Pe> {
        let bytes = std::fs::read(path).ok()?;
        let pe = u32_at(&bytes, 0x3c)? as usize;
        if bytes.get(pe..pe + 4)? != b"PE\0\0" {
            return None;
        }
        let nsec = u16_at(&bytes, pe + 6)? as usize;
        let optsz = u16_at(&bytes, pe + 20)? as usize;
        // `ImageBase` is 28 bytes into the 32-bit optional header.
        let image_base = u32_at(&bytes, pe + 24 + 28)?;
        let mut sections = Vec::with_capacity(nsec);
        for i in 0..nsec {
            let h = pe + 24 + optsz + i * 40;
            sections.push((
                u32_at(&bytes, h + 12)?,
                u32_at(&bytes, h + 8)?,
                u32_at(&bytes, h + 20)?,
                u32_at(&bytes, h + 16)?,
            ));
        }
        Some(Pe {
            bytes,
            image_base,
            sections,
        })
    }

    /// The file offset a virtual address maps to, if any section covers it.
    fn offset_of(&self, va: u32) -> Option<usize> {
        let rva = va.checked_sub(self.image_base)?;
        for &(sva, vsize, raw, rsize) in &self.sections {
            if rva >= sva && rva - sva < vsize.max(rsize) {
                return Some((raw + (rva - sva)) as usize);
            }
        }
        None
    }

    /// Every site the image's bytes hold that reaches `target`: a
    /// `call`, `jmp` or `jcc rel32` in the section that holds the target,
    /// found at every byte offset rather than by an instruction walk, and
    /// the target's address as four bytes at any offset of any section (a
    /// vtable slot, a table, a `push imm32`). A site inside another
    /// instruction's bytes is counted, so an answer errs toward referenced;
    /// what it cannot see is a rel8 jump and an address computed at run
    /// time (`tools/trace/report.py … refs`, which also reads the rel8
    /// neighbours and confirms each site against the listing).
    pub fn references(&self, target: u32) -> Vec<u32> {
        let mut out = Vec::new();
        let Some(rva) = target.checked_sub(self.image_base) else {
            return out;
        };
        let needle = target.to_le_bytes();
        for &(sva, vsize, raw, rsize) in &self.sections {
            let (raw, len) = (raw as usize, rsize.min(vsize) as usize);
            let Some(body) = self.bytes.get(raw..raw + len) else {
                continue;
            };
            let va = self.image_base + sva;
            for (i, w) in body.windows(4).enumerate() {
                if w == needle {
                    out.push(va + i as u32);
                }
            }
            if !(sva..sva + vsize).contains(&rva) {
                continue;
            }
            for i in 0..body.len() {
                let width = match (body[i], body.get(i + 1)) {
                    (0xE8 | 0xE9, _) => 1,
                    (0x0F, Some(0x80..=0x8F)) => 2,
                    _ => continue,
                };
                let Some(rel) = u32_at(body, i + width) else {
                    continue;
                };
                let site = va + i as u32;
                if site.wrapping_add(width as u32 + 4).wrapping_add(rel) == target {
                    out.push(site);
                }
            }
        }
        out.sort_unstable();
        out
    }

    /// `n` little-endian `i32`s at a virtual address.
    pub fn i32s(&self, va: u32, n: usize) -> Option<Vec<i32>> {
        let o = self.offset_of(va)?;
        (0..n)
            .map(|i| u32_at(&self.bytes, o + i * 4).map(|v| v as i32))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testenv::install_root;

    /// `move_x` and `move_y`, whole, against `sim::world::MOVE_289`.
    ///
    /// `rise.pdb` places `move_x` at `0002:97008` and `move_y` at
    /// `0002:95232` — section 2 is `.rdata`, based at `0x00AC5000`, so the
    /// virtual addresses are `0x00ADCAF0` and `0x00ADC400` — and types both
    /// as `int[441]`, the whole 21 × 21. The simulation walks the first 289
    /// of them (`Unit::think_fish`), and this is the check that the 289 it
    /// carries are the file's, **the typo at index 288 included**: nothing
    /// derives `move_y[288] = −16` from a rule, so if the table were
    /// regenerated from a clean generator this would be the test that
    /// noticed.
    #[test]
    fn the_move_table_is_the_executable_s_own() {
        let Some(root) = install_root() else {
            eprintln!("skipping: no install (set RON_INSTALL)");
            return;
        };
        let Some(pe) = Pe::open(&format!("{root}/riseofnations.exe")) else {
            panic!("riseofnations.exe is not a PE file");
        };
        let n = sim::world::MOVE_289.len();
        let xs = pe.i32s(0x00AD_CAF0, n).expect("move_x is in .rdata");
        let ys = pe.i32s(0x00AD_C400, n).expect("move_y is in .rdata");
        let file: Vec<(i32, i32)> = xs.into_iter().zip(ys).collect();
        assert_eq!(
            file,
            sim::world::MOVE_289.to_vec(),
            "move_x/move_y disagree with sim::world::MOVE_289"
        );
        assert_eq!(
            file[288],
            (-8, -16),
            "index 288 is the shipped table's typo, and it is load-bearing"
        );
    }
}
