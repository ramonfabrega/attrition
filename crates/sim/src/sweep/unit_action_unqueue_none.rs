//! `Unit::action_unqueue@005e1f20` with no current upgrade (answer -1, argument 0) under the emulator against the port (item 3391; tools/sweep_gen.py, obj with windows).

use sim::unit_unqueue_view::{self, UnitUnqueueView};

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

/// `Unit::action_unqueue@005e1f20`, run by `tools/emu/sweep_unit_action_unqueue_none.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p7 sparse table 0 (4 rows × 2 slice(s), row-major), p8..p22 the place(s), in flag order; callee 0x006e3140 (1 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0xd8 (3 args) stubbed; the word at `0xe85ddc` points at 24 random bytes less `0` (placed); the word at `p8+0` points at 16 random bytes less `0` (placed); the word at `p8+4` points at 16 random bytes less `0` (placed); the word at `p8+8` points at 16 random bytes less `0` (placed); the word at `p8+12` points at 16 random bytes less `0` (placed); the word at `p8+16` points at 16 random bytes less `0` (placed); the word at `p8+20` points at 16 random bytes less `0` (placed); the word at `0xc061fc` points at 32 random bytes less `0` (placed); the word at `p15+0x10` points at 24 random bytes less `0` (placed); the word at `p16+0` points at 576 random bytes less `0` (placed); the word at `p16+4` points at 576 random bytes less `0` (placed); the word at `p16+8` points at 576 random bytes less `0` (placed); the word at `p16+12` points at 576 random bytes less `0` (placed); the word at `p16+16` points at 576 random bytes less `0` (placed); the word at `p16+20` points at 576 random bytes less `0` (placed); slot +0xc laid as 0x004707e0 or its stub, a flag after the answers; stub 1 answered from [0, 1] (90%); stub 0 answers 0..5 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 0 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 4 rows 0x6eec apart at `0xe3a390`, slices +0xa10:0x18, +0x5a22:0x10 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_action_unqueue_none.py") else {
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
            // `a` is `p, s0 (current_upgrade), s1 (the type's slot 0xc), s2 (slot 0xd8), slot flag`.
            let _ = &c;
            let mut view = Rows {
                a,
                m,
                p,
                log: &mut log,
            };
            unit_unqueue_view::unit_unqueue_steps(&mut view, a[0]);
            if log.is_empty() {
                log.push("-".to_string());
            }
            0
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(
            log, want_log,
            "action_unqueue — the recorded calls, row `{line}`"
        );
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_unqueue — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_unqueue — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "action_unqueue — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Unit::action_unqueue@005e1f20: {n} rows agree");
}

// hand: view
/// The row's windows as the view: `p0..p7` are the four leaders' counters and shorts, `p9..p14` the
/// type records, `p17..p22` the unit records.
struct Rows<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
}

impl Rows<'_> {
    fn row(&self) -> usize {
        usize::from(self.m[9])
    }
    fn counters(&mut self) -> &mut Vec<u8> {
        let r = self.row();
        &mut self.p[2 * r]
    }
    fn shorts(&mut self) -> &mut Vec<u8> {
        let r = self.row();
        &mut self.p[2 * r + 1]
    }
}

impl UnitUnqueueView for Rows<'_> {
    fn queued(&mut self) -> i16 {
        i16::from_le_bytes([self.m[0xa0], self.m[0xa1]])
    }
    fn set_queued(&mut self, n: i16) {
        self.m[0xa0..0xa2].copy_from_slice(&n.to_le_bytes());
    }
    fn clear_word_64(&mut self) {
        wr(self.m, 0x64, 0);
    }
    fn current_upgrade(&mut self) -> i32 {
        let r = self.row();
        self.log
            .push(format!("s0:r{r}+0x0,{}", unit_unqueue_view::FIGHTERBOMBER));
        self.a[1]
    }
    fn tally(&mut self, idx: i32) -> u16 {
        let o = 2 * idx as usize;
        let s = self.shorts();
        u16::from_le_bytes([s[o], s[o + 1]])
    }
    fn set_tally(&mut self, idx: i32, v: u16) {
        let o = 2 * idx as usize;
        self.shorts()[o..o + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn type_is_unit(&mut self, idx: i32) -> bool {
        if self.a[4] != 0 {
            let t = rd(&self.p[9 + idx as usize], 4);
            (0x32..0x19e).contains(&t)
        } else {
            self.a[2] != 0
        }
    }
    fn unit_word_1e8(&mut self, idx: i32) -> i32 {
        rd(&self.p[17 + idx as usize], 0x1e8)
    }
    fn unit_word_40(&mut self, idx: i32) -> i32 {
        rd(&self.p[17 + idx as usize], 0x40)
    }
    fn unit_word_218(&mut self, idx: i32) -> i32 {
        rd(&self.p[17 + idx as usize], 0x218)
    }
    fn counter(&mut self, off: usize) -> i32 {
        rd(self.counters(), off - 0xa10)
    }
    fn set_counter(&mut self, off: usize, v: i32) {
        wr(self.counters(), off - 0xa10, v);
    }
    fn type_slot_d8(&mut self, idx: i32) {
        let who = i32::from(self.m[9]);
        self.log.push(format!("s2:p{}+0x0,{who},-1,-1", 9 + idx));
    }
}
// end hand
