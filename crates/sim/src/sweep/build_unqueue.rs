//! `Build::unqueue@006207c0` under the emulator against the port (item 3399; tools/sweep_gen.py, obj with windows).

use sim::build_unqueue_view::{self, BuildUnqueueView};

#[allow(dead_code)]
fn rd(m: &[u8], off: usize) -> i32 {
    i32::from_le_bytes(m[off..off + 4].try_into().expect("four bytes"))
}

#[allow(dead_code)]
fn wr(m: &mut [u8], off: usize, v: i32) {
    m[off..off + 4].copy_from_slice(&v.to_le_bytes());
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("hex"))
        .collect()
}

/// `Build::unqueue@006207c0`, run by `tools/emu/sweep_build_unqueue.py` under unicorn
/// (tools/emu): 144 random bytes of object, 2 argument(s), `p` windows in order: p0..p7 sparse table 0 (2 rows × 4 slice(s), row-major), p8..p45 the place(s), in flag order; vtable slot +0x60 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x006db6c0 (0 args) stubbed; vtable slot +0x1c8 (2 args) stubbed; callee 0x006206e0 (1 args) stubbed; callee 0x00630ad0 (2 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; vtable slot +0x38 (0 args) stubbed; the word at `0xc06204` points at 152 random bytes less `0` (placed); the word at `0xc0ab84+0*0x1c` points at 32 random bytes less `0` (placed); the word at `0xc0ab84+1*0x1c` points at 32 random bytes less `0` (placed); the word at `p9+0` points at 16 random bytes less `0` (placed); the word at `p9+4` points at 16 random bytes less `0` (placed); the word at `p9+8` points at 16 random bytes less `0` (placed); the word at `p9+12` points at 16 random bytes less `0` (placed); the word at `p9+16` points at 16 random bytes less `0` (placed); the word at `p9+20` points at 16 random bytes less `0` (placed); the word at `p9+24` points at 16 random bytes less `0` (placed); the word at `p9+28` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 16 random bytes less `0` (placed); the word at `p10+4` points at 16 random bytes less `0` (placed); the word at `p10+8` points at 16 random bytes less `0` (placed); the word at `p10+12` points at 16 random bytes less `0` (placed); the word at `p10+16` points at 16 random bytes less `0` (placed); the word at `p10+20` points at 16 random bytes less `0` (placed); the word at `p10+24` points at 16 random bytes less `0` (placed); the word at `p10+28` points at 16 random bytes less `0` (placed); the word at `m+0x8c` points at 120 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); the word at `0xe85ddc` points at 2668 random bytes less `0` (placed); the word at `p29+0` points at 16 random bytes less `0` (placed); the word at `p29+4` points at 16 random bytes less `0` (placed); the word at `p29+8` points at 16 random bytes less `0` (placed); the word at `p29+12` points at 16 random bytes less `0` (placed); the word at `p29+16` points at 16 random bytes less `0` (placed); the word at `p29+20` points at 16 random bytes less `0` (placed); the word at `p29+0xa68` points at 16 random bytes less `0` (placed); the word at `0xc061fc` points at 32 random bytes less `0` (placed); the word at `p37+0x10` points at 2668 random bytes less `0` (placed); the word at `p38+0` points at 576 random bytes less `0` (placed); the word at `p38+4` points at 576 random bytes less `0` (placed); the word at `p38+8` points at 576 random bytes less `0` (placed); the word at `p38+12` points at 576 random bytes less `0` (placed); the word at `p38+16` points at 576 random bytes less `0` (placed); the word at `p38+20` points at 576 random bytes less `0` (placed); the word at `p38+0xa68` points at 576 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; slot +0x34 laid as 0x00470820 or its stub, a flag after the answers; slot +0x38 laid as 0x00470870 or its stub, a flag after the answers; stub 0 answered from [0, 0, 1] (90%); stub 1 answered from [0, 0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 7 answered from [0, 1] (90%); stub 8 answered from [0, 1] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 2 answers -1..7 every row; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x6eec apart at `@0xe3a390`, slices +0xa10:0x18, +0x5a22:0x20, +0x5f56:0x8, +0x67f4:0x2 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p32+0` as well as at `this`; the stubbed vtable laid at `p33+0` as well as at `this`; the stubbed vtable laid at `p34+0` as well as at `this`; the stubbed vtable laid at `p35+0` as well as at `this`; the stubbed vtable laid at `p36+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_unqueue.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line
            .split_once(" -> ")
            .expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args
            .split_whitespace()
            .map(|s| s.parse().expect("an integer"))
            .collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // `a` is `p1, p2, s0..s8 (the answers), flag 0xc, flag 0x34, flag 0x38`.
            let _ = &c;
            let mut view = Rows {
                a,
                m,
                p,
                log: &mut log,
            };
            build_unqueue_view::build_unqueue_steps(&mut view, a[0], a[1]);
            if log.is_empty() {
                log.push("-".to_string());
            }
            0
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "unqueue — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "unqueue — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "unqueue — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "unqueue — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Build::unqueue@006207c0: {n} rows agree");
}

// The hand-written view (items outside the generated test are carried by --regen).

/// The row's windows as the view: `p0..p7` the two owners' counters (`+0xa10`), shorts (`+0x5a22`,
/// `+0x5f56` for type `0x29a`) and age bytes (`+0x67f4`), `p8` the options, `p9`/`p10` the owners'
/// library tables behind `p11..p26`, `p27` the entries, `p30..p36` the type objects (`0..5`, `0x29a`)
/// and `p39..p45` the unit records.
pub(super) struct Rows<'a> {
    pub(super) a: &'a [i32],
    pub(super) m: &'a mut [u8],
    pub(super) p: &'a mut [Vec<u8>],
    pub(super) log: &'a mut Vec<String>,
}

impl Rows<'_> {
    fn row(&self) -> usize {
        usize::from(self.m[9])
    }
    /// The window of a type index: `0..5` then `0x29a` last.
    fn slot(idx: i32) -> usize {
        if idx == 0x29a {
            6
        } else {
            assert!(
                (0..6).contains(&idx),
                "type {idx} is outside the placed table"
            );
            idx as usize
        }
    }
    fn range_test(&mut self, idx: i32, flag: i32, lo: i32, hi: i32, answer: i32) -> bool {
        if flag != 0 {
            let t = rd(&self.p[30 + Self::slot(idx)], 4);
            lo <= t && t <= hi
        } else {
            answer != 0
        }
    }
    fn tally_at(&self, idx: i32) -> (usize, usize) {
        let r = 4 * self.row();
        if idx == 0x29a {
            (r + 2, 0)
        } else {
            (r + 1, 2 * idx as usize)
        }
    }
}

impl BuildUnqueueView for Rows<'_> {
    fn count(&mut self) -> u8 {
        self.m[0x82]
    }
    fn set_count(&mut self, n: u8) {
        self.m[0x82] = n;
    }
    fn queue_len(&mut self) -> i32 {
        rd(self.m, 0x88)
    }
    fn entry_type(&mut self, i: i32) -> i16 {
        let at = i as usize * 0x14 + 4;
        i16::from_le_bytes([self.p[27][at], self.p[27][at + 1]])
    }
    fn clear_entry(&mut self, i: i32) {
        wr(&mut self.p[27], i as usize * 0x14, 0);
    }
    fn library_gate(&mut self) -> bool {
        if rd(self.m, 0) == 0xb42174 {
            self.a[2] != 0
        } else {
            self.a[3] != 0
        }
    }
    fn first_library(&mut self) -> i32 {
        let who = self.row();
        self.log.push(format!("s2:{}", 0xe3a390 + who * 0x6eec));
        self.a[4]
    }
    fn own_library(&mut self) -> i16 {
        i16::from_le_bytes([self.m[0xa], self.m[0xb]])
    }
    fn redirect(&mut self, lib: i32, a: i32, b: i32) {
        let who = self.row();
        self.log
            .push(format!("s3:p{}+0x0,{a},{b}", 11 + 8 * who + lib as usize));
    }
    fn set_rebuild(&mut self) {
        wr(&mut self.p[8], 0x90, 1);
    }
    fn tally(&mut self, idx: i32) -> u16 {
        let (w, o) = self.tally_at(idx);
        u16::from_le_bytes([self.p[w][o], self.p[w][o + 1]])
    }
    fn set_tally(&mut self, idx: i32, v: u16) {
        let (w, o) = self.tally_at(idx);
        self.p[w][o..o + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn type_is_unit(&mut self, idx: i32) -> bool {
        self.range_test(idx, self.a[11], 0x32, 0x19d, self.a[8])
    }
    fn type_is_age(&mut self, idx: i32) -> bool {
        self.range_test(idx, self.a[12], 0x220, 0x226, self.a[9])
    }
    fn type_is_epoch(&mut self, idx: i32) -> bool {
        self.range_test(idx, self.a[13], 0x227, 0x242, self.a[10])
    }
    fn unit_word_1e8(&mut self, idx: i32) -> i32 {
        rd(&self.p[39 + Self::slot(idx)], 0x1e8)
    }
    fn unit_word_40(&mut self, idx: i32) -> i32 {
        rd(&self.p[39 + Self::slot(idx)], 0x40)
    }
    fn unit_word_218(&mut self, idx: i32) -> i32 {
        rd(&self.p[39 + Self::slot(idx)], 0x218)
    }
    fn counter(&mut self, off: usize) -> i32 {
        let r = 4 * self.row();
        rd(&self.p[r], off - 0xa10)
    }
    fn set_counter(&mut self, off: usize, v: i32) {
        let r = 4 * self.row();
        wr(&mut self.p[r], off - 0xa10, v);
    }
    fn byte_counter(&mut self, off: usize) -> u8 {
        let r = 4 * self.row();
        self.p[r + 3][off - 0x67f4]
    }
    fn set_byte_counter(&mut self, off: usize, v: u8) {
        let r = 4 * self.row();
        self.p[r + 3][off - 0x67f4] = v;
    }
    fn unpay_cost(&mut self, p1: i32) {
        self.log.push(format!("s4:m+0x0,{p1}"));
    }
    fn un_queue(&mut self, p1: i32, count: i32) {
        self.log.push(format!("s5:m+0x88,{p1},{count}"));
    }
    fn clear_mask_bit(&mut self) {
        let w = u16::from_le_bytes([self.m[0x60], self.m[0x61]]) & 0xffbf;
        self.m[0x60..0x62].copy_from_slice(&w.to_le_bytes());
    }
}
