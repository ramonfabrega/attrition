//! `Unit::set_type@00612fa0` under the emulator against the port (item 3402; tools/sweep_gen.py, obj with windows).

use sim::unit_set_type_view::{self, UnitSetTypeView};

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

/// `Unit::set_type@00612fa0`, run by `tools/emu/sweep_unit_set_type_normal.py` under unicorn
/// (tools/emu): 256 random bytes of object, 2 argument(s), `p` windows in order: p0..p1 sparse table 0 (2 rows × 1 slice(s), row-major), p2..p5 sparse table 1 (2 rows × 2 slice(s), row-major), p6..p31 the place(s), in flag order, p32..p37 the stubs' answer window(s); the global cell 0x00c061f8 points at 32 random bytes; the global cell 0x00c061fc points at 32 random bytes; the global cell 0x00c8d840 points at 64 random bytes; the global cell 0x00cae5fc points at 64 random bytes; the global cell 0x00c061ec points at 32 random bytes; 12 random bytes at 0x00c8d844 (each word in 0..3); 4 random bytes at 0x00e7fe4c (each word in 33752069..33752069); vtable slot +0xe8 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0xc8 (0 args) stubbed; vtable slot +0xd0 (0 args) stubbed; vtable slot +0xcc (0 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0x130 (0 args) stubbed; vtable slot +0x15c (1 args) stubbed; vtable slot +0x160 (0 args) stubbed; callee 0x006e0dd0 (3 args) stubbed; callee 0x00659410 (7 args) stubbed; callee 0x005db590 (0 args) stubbed; callee 0x0046ea60 (1 args) stubbed; callee 0x00422300 (1 args) stubbed; callee 0x0046de80 (0 args) stubbed; callee 0x005db6b0 (1 args) stubbed; callee 0x00662450 (2 args) stubbed; callee 0x005f96b0 (4 args) stubbed; callee 0x006054c0 (0 args) stubbed; callee 0x006055c0 (0 args) stubbed; callee 0x005e2920 (0 args) stubbed; callee 0x005d86f0 (3 args) stubbed; callee 0x0073acc0 (2 args) stubbed; callee 0x0073ad40 (2 args) stubbed; the word at `p0+0` points at 8 random bytes less `0` (placed); the word at `p6+0` points at 64 random bytes less `0` (placed); the word at `p6+4` points at 64 random bytes less `0` (placed); the word at `p1+0` points at 8 random bytes less `0` (placed); the word at `p9+0` points at 64 random bytes less `0` (placed); the word at `p9+4` points at 64 random bytes less `0` (placed); the word at `m+0xf4` points at 24 random bytes less `0` (placed); the word at `p12+0` points at 176 random bytes less `0` (placed); the word at `p12+4` points at 176 random bytes less `0` (placed); the word at `p12+8` points at 176 random bytes less `0` (placed); the word at `p12+12` points at 176 random bytes less `0` (placed); the word at `p12+16` points at 176 random bytes less `0` (placed); the word at `p12+20` points at 176 random bytes less `0` (placed); the word at `c0+0x10` points at 12 random bytes less `0` (placed); the word at `p19+0` points at 800 random bytes less `0` (placed); the word at `p19+4` points at 800 random bytes less `0` (placed); the word at `p19+8` points at 800 random bytes less `0` (placed); the word at `c1+0x10` points at 12 random bytes less `0` (placed); the word at `p23+0` points at 800 random bytes less `0` (placed); the word at `p23+4` points at 800 random bytes less `0` (placed); the word at `p23+8` points at 800 random bytes less `0` (placed); the word at `m+0x18` points at 800 random bytes less `0` (placed); the word at `p7+0x18` points at 800 random bytes less `0` (placed); the word at `p8+0x18` points at 800 random bytes less `0` (placed); the word at `p10+0x18` points at 800 random bytes less `0` (placed); the word at `p11+0x18` points at 800 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0xc8 laid as 0x0046ce70 or its stub, a flag after the answers; slot +0xd0 laid as 0x0046ce90 or its stub, a flag after the answers; slot +0xcc laid as 0x0046ce80 or its stub, a flag after the answers; stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); stub 4 answered from [0, 1] (90%); stub 5 answered from [0, 1] (90%); stub 6 answered from [0, 1] (90%); stub 14 answers a window of its own to each call, in call order (176 bytes, ...); the word at `c3+0x20` laid as the address of `p32` every row; the word at `c3+0x24` laid as the address of `p33` every row; the word at `c3+0x28` laid as the address of `p34` every row; the word at `c3+0x2c` laid as the address of `p35` every row; the word at `c3+0x30` laid as the address of `p36` every row; the word at `c3+0x34` laid as the address of `p37` every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 1 (a token a row, `ecx` `r+OFF`, then the calls outside it); stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `, split by the rows of sparse table 1 (a token a row, `ecx` `r+OFF`, then the calls outside it); the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); 2 rows 0x1c apart at `@0xc0aec0`, slices +0x0:0x4 (`p` windows after the pointed buffers, before the places); 2 rows 0x6eec apart at `@0xe3a390`, slices +0x93c:0x8, +0x980:0x4 (`p` windows after the pointed buffers, before the places); the stubbed vtable laid at `p27+0` as well as at `this`; the stubbed vtable laid at `p28+0` as well as at `this`; the stubbed vtable laid at `p29+0` as well as at `this`; the stubbed vtable laid at `p30+0` as well as at `this`; the stubbed vtable laid at `p31+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 3000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_set_type_normal.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(7);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // hand: port
            // `a[0]` the type, `a[1]` the mode, `a[2 + k]` stub k's answer, `a[26 + j]` the slot
            // flags (0xe8, 0xb8, 0xc8, 0xd0, 0xcc: 1 = the image's own slot).
            let _ = &mut *c;
            let mut view = Rows::new(a, m, c, p, &mut log);
            unit_set_type_view::set_type_steps(&mut view, a[0], a[1]);
            0
            // end hand
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "set_type — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "set_type — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "set_type — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[8 + i], "set_type — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 3000, "only {n} rows");
    eprintln!("Unit::set_type@00612fa0: {n} rows agree");
}

// hand: view
/// The row's windows as the view. `m` is the unit; `c[0]`/`c[1]` the type tables' records, `c[2]`
/// the recycler's list, `c[3]` the `div_3_table`, `c[4]` the game record (`+0xc` the version),
/// `c[5]` the recycler's `size`/`length`/`increment`, `c[6]` the `xpack_rtm_version`; `p2..p5` the
/// leaders' counters (two slices a leader), `p7/p8/p10/p11` the table's units and `p28..p31`
/// their types, `p12` the guy array and `p13..p18` its guys, `p19..p22` the object types,
/// `p24..p26` the unit types, `p27` the unit's own type, `p32..p37` the guys the recycler pops
/// (their addresses laid in `c[3]` at `+0x20`).
struct Rows<'a> {
    a: &'a [i32],
    m: &'a mut [u8],
    c: &'a mut [Vec<u8>],
    p: &'a mut [Vec<u8>],
    log: &'a mut Vec<String>,
    /// The window of the unit's current type record.
    cur: usize,
    /// Guy addresses and their windows: the array's six, then the six pops'.
    guys: Vec<(u32, usize)>,
    /// The pool length the last push left in `ecx`.
    last_len: i32,
    /// `ecx` as the next `Recycler::pop` is called.
    pop_ecx: String,
    /// Pops made.
    pops: usize,
    /// `ecx` as the supply calls are made.
    ecx: String,
}

impl<'a> Rows<'a> {
    fn new(
        a: &'a [i32],
        m: &'a mut [u8],
        c: &'a mut [Vec<u8>],
        p: &'a mut [Vec<u8>],
        log: &'a mut Vec<String>,
    ) -> Self {
        let mut guys = Vec::new();
        for k in 0..6 {
            guys.push((rd(&p[12], 4 * k) as u32, 13 + k));
        }
        for k in 0..6 {
            guys.push((rd(&c[3], 0x20 + 4 * k) as u32, 32 + k));
        }
        Rows {
            a,
            m,
            c,
            p,
            log,
            cur: 27,
            guys,
            last_len: 0,
            pop_ecx: String::new(),
            pops: 0,
            ecx: String::new(),
        }
    }
    fn who(&self) -> usize {
        usize::from(self.m[9])
    }
    fn idx(&self) -> i32 {
        i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
    }
    /// The table's unit for this one, and its type's window.
    fn table(&self) -> (usize, usize) {
        let k = 2 * self.who() + usize::try_from(self.idx()).expect("idx");
        ([7, 8, 10, 11][k], [28, 29, 30, 31][k])
    }
    /// The window a guy address names; `None` for the null page, where the original's writes
    /// land unseen.
    fn win_of(&self, addr: u32) -> Option<usize> {
        if addr == 0 {
            return None;
        }
        Some(
            self.guys
                .iter()
                .find(|g| g.0 == addr)
                .map(|g| g.1)
                .unwrap_or_else(|| panic!("no window at {addr:#x}")),
        )
    }
    /// The recorded name of a guy address: its window, or `0`.
    fn guy_name(&self, addr: u32) -> String {
        self.win_of(addr)
            .map_or_else(|| "0".to_string(), |w| format!("p{w}+0x0"))
    }
    /// The guy list's entry `i`.
    fn list(&self, i: i32) -> u32 {
        rd(&self.p[12], 4 * usize::try_from(i).expect("slot")) as u32
    }
    /// Stub `stub`'s answer.
    fn answer(&self, stub: usize) -> bool {
        self.a[2 + stub] != 0
    }
    /// Whether slot flag `j` (0xe8, 0xb8, 0xc8, 0xd0, 0xcc) laid the image's own slot.
    fn flag(&self, j: usize) -> bool {
        self.a[26 + j] != 0
    }
}

impl UnitSetTypeView for Rows<'_> {
    fn is_captain(&mut self) -> bool {
        if self.flag(0) {
            (rd(self.m, 0x8c) as u32) >> 31 != 0
        } else {
            self.log.push("s0:m+0x0".to_string());
            self.answer(0)
        }
    }
    fn decoy_bit(&mut self) -> bool {
        self.m[0x68] & 1 != 0
    }
    fn type_pop(&mut self) -> i32 {
        rd(&self.p[self.cur], 0x2f0)
    }
    fn table_unit_is_134(&mut self) -> bool {
        let (u, t) = self.table();
        if self.flag(1) {
            self.log.push(format!("s5:p{t}+0x0,308,0"));
            self.answer(5)
        } else {
            self.log.push(format!("s1:p{u}+0x0,308,0"));
            self.answer(1)
        }
    }
    fn table_unit_is_gov_hero(&mut self) -> bool {
        let (u, t) = self.table();
        if self.flag(2) {
            rd(&self.p[t], 0x2b4) & 0x4000000 != 0
        } else {
            self.log.push(format!("s2:p{u}+0x0"));
            self.answer(2)
        }
    }
    fn old_type_index(&mut self) -> i32 {
        rd(&self.p[self.cur], 4)
    }
    fn track_unit_type(&mut self, ty: i32, delta: i32) {
        let (w, i) = (self.who(), self.idx());
        self.log.push(format!("s9:r{w}+0x0,{ty},{delta},{i}"));
    }
    fn leader_totals(&mut self, delta: i32, pop: i32) {
        let row = 2 + 2 * self.who();
        let control = rd(&self.p[row], 4).wrapping_add(delta.wrapping_mul(pop));
        wr(&mut self.p[row], 4, control);
        let active = rd(&self.p[row], 0).wrapping_add(delta);
        wr(&mut self.p[row], 0, active);
    }
    fn is_caravan(&mut self) -> bool {
        if self.flag(3) {
            self.log.push(format!("s6:p{}+0x0", self.cur));
            self.answer(6)
        } else {
            self.log.push("s3:m+0x0".to_string());
            self.answer(3)
        }
    }
    fn leader_caras(&mut self, delta: i32) {
        let row = 3 + 2 * self.who();
        let caras = rd(&self.p[row], 0).wrapping_add(delta);
        wr(&mut self.p[row], 0, caras);
    }

    fn guy_total(&mut self) -> i32 {
        rd(self.m, 0xe8)
    }
    fn guy_mark(&mut self) -> i32 {
        i32::from(self.m[0xb5] as i8)
    }
    fn set_guy_mark(&mut self, n: i32) {
        self.m[0xb5] = n as u8;
    }
    fn has_type(&mut self) -> bool {
        rd(self.m, 0x18) != 0
    }
    fn squad_size(&mut self, ty: i32) -> i32 {
        rd(&self.p[24 + usize::try_from(ty).expect("type")], 0x304)
    }
    fn crew_size(&mut self, ty: i32) -> i32 {
        rd(&self.p[24 + usize::try_from(ty).expect("type")], 0x30c)
    }
    fn kill_and_recycle(&mut self, i: i32, entry: Option<i32>) {
        let ecx = entry.unwrap_or(self.last_len);
        let (w, x) = (self.who(), self.idx());
        self.log.push(format!("s10:{ecx},{w},{x},{i},0,-1,0,1"));
        let slot = 4 * usize::try_from(i).expect("slot");
        let ptr = rd(&self.p[12], slot) as u32;
        let g = if ptr == 0xffff_ff17 { 0 } else { ptr };
        let name = self.guy_name(g);
        self.log.push(format!("s11:{name}"));
        let len = rd(&self.c[5], 4);
        if len >= rd(&self.c[5], 0) {
            self.log
                .push(format!("s12:{len},{}", i32::from(self.c[5][8] as i8)));
        }
        let len = rd(&self.c[5], 4);
        wr(
            &mut self.c[2],
            4 * usize::try_from(len).expect("len"),
            g as i32,
        );
        wr(&mut self.c[5], 4, len.wrapping_add(1));
        self.last_len = len;
        wr(&mut self.p[12], slot, 0);
    }
    fn guy_capacity(&mut self) -> i32 {
        rd(self.m, 0xec)
    }
    fn guy_hint(&mut self) -> i32 {
        i32::from(i16::from_le_bytes([self.m[0xf0], self.m[0xf1]]))
    }
    fn grow_guys(&mut self, n: i32) {
        self.log.push(format!("s13:m+0xe4,{n}"));
    }
    fn zero_guy_slot(&mut self, i: i32) {
        wr(&mut self.p[12], 4 * usize::try_from(i).expect("slot"), 0);
    }
    fn set_guy_total(&mut self, n: i32) {
        wr(self.m, 0xe8, n);
        self.pop_ecx = n.to_string();
    }
    fn pop_guy(&mut self, i: i32) {
        self.log.push(format!("s14:{}", self.pop_ecx));
        let g = self.guys[6 + self.pops.min(5)];
        self.pops += 1;
        wr(
            &mut self.p[12],
            4 * usize::try_from(i).expect("slot"),
            g.0 as i32,
        );
        let name = format!("p{}+0x0", g.1);
        self.log.push(format!("s11:{name}"));
        self.pop_ecx = name;
    }
    fn install_type(&mut self, ty: i32) {
        let t = usize::try_from(ty).expect("type");
        let ptr = rd(&self.p[19], 4 * t);
        wr(self.m, 0x18, ptr);
        self.cur = 20 + t;
    }
    fn init_kept(&mut self, i: i32, ty: i32) {
        let ptr = self.list(i);
        if let Some(g) = self.win_of(ptr) {
            wr(&mut self.p[g], 8, ty);
        }
        let name = self.guy_name(ptr);
        self.log.push(format!("s15:{name},1"));
        if let Some(g) = self.win_of(ptr) {
            self.p[g][0xa2] = i as u8;
        }
    }
    fn init_new(&mut self, i: i32, ty: i32) {
        let ptr = self.list(i);
        if let Some(g) = self.win_of(ptr) {
            wr(&mut self.p[g], 8, ty);
            self.p[g][0xa2] = i as u8;
            self.p[g][0xa1] = self.m[9];
            let (lo, hi) = (self.m[0xa], self.m[0xb]);
            self.p[g][0x8c] = lo;
            self.p[g][0x8d] = hi;
        }
        let name = self.guy_name(ptr);
        self.log.push(format!("s15:{name},1"));
    }

    fn sub_object_set_type(&mut self, ty: i32, mode: i32) {
        self.log.push(format!("s16:m+0x0,{ty},{mode}"));
    }
    fn is_ceo(&mut self) -> bool {
        if self.flag(1) {
            self.log.push(format!("s5:p{}+0x0,357,1", self.cur));
            self.answer(5)
        } else {
            self.log.push("s1:m+0x0,357,1".to_string());
            self.answer(1)
        }
    }
    fn flags6c(&mut self) -> i32 {
        rd(self.m, 0x6c)
    }
    fn set_flags6c(&mut self, f: i32) {
        wr(self.m, 0x6c, f);
    }
    fn update_ceo_position(&mut self) {
        let idx = |off: usize, m: &[u8]| ((rd(m, off) ^ 0x63637) >> 6) as usize;
        let x = rd(&self.c[3], 4 * idx(0x10, self.m));
        let y = rd(&self.c[3], 4 * idx(0x14, self.m));
        self.log.push(format!("s17:m+0x0,-1,-1,{x},{y}"));
    }
    fn is_77(&mut self) -> bool {
        if self.flag(1) {
            self.log.push(format!("s5:p{}+0x0,119,0", self.cur));
            self.answer(5)
        } else {
            self.log.push("s1:m+0x0,119,0".to_string());
            self.answer(1)
        }
    }
    fn slot_15c(&mut self) {
        self.log.push("s7:m+0x0,0".to_string());
    }
    fn slot_160(&mut self) {
        self.log.push("s8:m+0x0".to_string());
    }
    fn update_armor(&mut self) {
        self.log.push("s18:m+0x0".to_string());
    }
    fn update_speed(&mut self) {
        self.log.push("s19:m+0x0".to_string());
    }
    fn update_gpiece(&mut self) {
        self.log.push("s20:m+0x0".to_string());
    }
    fn seat_guy(&mut self, i: i32) {
        let name = self.guy_name(self.list(i));
        let (x, y) = (rd(self.m, 0x10) ^ 0x63637, rd(self.m, 0x14) ^ 0x63637);
        self.log.push(format!("s21:{name},{x},{y},1"));
    }
    fn version_newer(&mut self) -> bool {
        let v = rd(&self.c[4], 0xc) as u32;
        let x = rd(&self.c[6], 0) as u32;
        let (vb, xb) = (v.to_le_bytes(), x.to_le_bytes());
        // `ecx` is the executable's own word until byte 2 is compared, then `version >> 16`.
        if vb[0] != xb[0] || vb[1] != xb[1] {
            self.ecx = x.to_string();
            return if vb[0] != xb[0] {
                vb[0] > xb[0]
            } else {
                vb[1] > xb[1]
            };
        }
        self.ecx = (v >> 16).to_string();
        (vb[2], vb[3]) > (xb[2], xb[3])
    }
    fn is_supply(&mut self) -> bool {
        if self.flag(4) {
            rd(&self.p[self.cur], 0x2b8) & 0x40 != 0
        } else {
            self.log.push("s4:m+0x0".to_string());
            self.ecx = "m+0x0".to_string();
            self.answer(4)
        }
    }
    fn supply_id(&mut self) -> i32 {
        i32::from(i16::from_le_bytes([self.m[0x84], self.m[0x85]]))
    }
    fn close_supply(&mut self, id: i32) {
        let w = self.who();
        self.log.push(format!("s22:{},{w},{id}", self.ecx));
        self.m[0x84..0x86].copy_from_slice(&(-1i16).to_le_bytes());
    }
    fn open_supply(&mut self) {
        let (w, i) = (self.who(), self.idx());
        self.log.push(format!("s23:{},{w},{i}", self.ecx));
        let h = self.a[2 + 23] as i16;
        self.m[0x84..0x86].copy_from_slice(&h.to_le_bytes());
    }
}
// end hand
