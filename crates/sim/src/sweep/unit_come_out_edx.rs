//! `Unit::come_out@00617c10` under the emulator against the port (item 3362; tools/sweep_gen.py, obj with windows).

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

/// `Unit::come_out@00617c10`, run by `tools/emu/sweep_unit_come_out_edx.py` under unicorn
/// (tools/emu): 256 random bytes of object, 1 argument(s), `p` windows in order: p0..p32 the place(s), in flag order; the global cell 0x00c0618c points at 256 random bytes; the global cell 0x00c061f0 points at 256 random bytes; the global cell 0x00c06218 points at 64 random bytes; 16 random bytes at 0x00e4f054; vtable slot +0xe8 (0 args) stubbed; vtable slot +0xe4 (0 args) stubbed; vtable slot +0x1c (0 args) stubbed; vtable slot +0x20 (0 args) stubbed; vtable slot +0x34 (0 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00651a80 (1 args) stubbed; callee 0x0061de70 (16 args) stubbed; callee 0x006e3140 (1 args) stubbed; callee 0x0065e0c0 (7 args) stubbed; callee 0x0046f180 (0 args) stubbed; callee 0x0046f0c0 (0 args) stubbed; callee 0x00714fd0 (3 args) stubbed; callee 0x0092d130 (0 args) stubbed; callee 0x008544a0 (4 args) stubbed; callee 0x006480f0 (0 args) stubbed; callee 0x005f8d20 (4 args) stubbed; the word at `c0+0x14+0x1c*3` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 128 random bytes less `0` (placed); the word at `p0+4` points at 128 random bytes less `0` (placed); the word at `p0+8` points at 128 random bytes less `0` (placed); the word at `p0+12` points at 128 random bytes less `0` (placed); the word at `m+0xe0` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 256 random bytes less `0` (placed); the word at `p5+4` points at 256 random bytes less `0` (placed); the word at `p5+8` points at 256 random bytes less `0` (placed); the word at `p5+12` points at 256 random bytes less `0` (placed); the word at `0xc0af14` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 256 random bytes less `0` (placed); the word at `p10+4` points at 256 random bytes less `0` (placed); the word at `p10+8` points at 256 random bytes less `0` (placed); the word at `p10+12` points at 256 random bytes less `0` (placed); the word at `m+0xf4` points at 12 random bytes less `0` (placed); the word at `p15+0` points at 224 random bytes less `0` (placed); the word at `p15+4` points at 224 random bytes less `0` (placed); the word at `p15+8` points at 224 random bytes less `0` (placed); the word at `m+0x18` points at 608 random bytes less `0` (placed); the word at `p1+0x18` points at 608 random bytes less `0` (placed); the word at `p2+0x18` points at 608 random bytes less `0` (placed); the word at `p3+0x18` points at 608 random bytes less `0` (placed); the word at `p4+0x18` points at 608 random bytes less `0` (placed); the word at `p11+0x18` points at 608 random bytes less `0` (placed); the word at `p12+0x18` points at 608 random bytes less `0` (placed); the word at `p13+0x18` points at 608 random bytes less `0` (placed); the word at `p14+0x18` points at 608 random bytes less `0` (placed); the word at `p6+0xbc` points at 16 random bytes less `0` (placed); the word at `p7+0xbc` points at 16 random bytes less `0` (placed); the word at `p8+0xbc` points at 16 random bytes less `0` (placed); the word at `p9+0xbc` points at 16 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; stub 0 answered from [0, 1, 1, 0] (90%); stub 2 answered from [0, 1, 1] (90%); stub 3 answered from [0, 1, 1, 1] (90%); stub 6 answered from [0, 0, 1, 1, 0] (90%); stub 7 answered from [0, 0, 1, 1, 0] (90%); stub 10 answered from [0, 0, 1, 1, 0] (90%); stub 13 answered from [0, 0, 1] (90%); stub 5 answers `p6+j`, the data of the object its `ecx` is in `p1+j` (j < 4), linked at `+0xf8`; stub 0 answers 3 values in call order, printed after the answers; stub 1 answers 2 values in call order, printed after the answers; stub 2 answers 2 values in call order, printed after the answers; stub 3 answers 3 values in call order, printed after the answers; stub 6 answers 4 values in call order, printed after the answers; stub 7 answers 4 values in call order, printed after the answers; stub 9 answers 1 values in call order, printed after the answers; stub 10 answers 6 values in call order, printed after the answers; stub 12 answers 1 values in call order, printed after the answers; stub 13 answers 1 values in call order, printed after the answers; stub 1 answers 0..3 every row; stub 9 answers -1..3 every row; stub 12 answers -5..-1 every row; stub 16 records its `ecx`, `edx` and arguments, a window's pointer by its name, after ` # `; stub 9 writes a value in [3] through its argument 0, printed after the answers (`o` in a place); stub 10 writes a value in 0..16383 through its argument 2, printed after the answers (`o` in a place); stub 10 writes a value in 0..16383 through its argument 3, printed after the answers (`o` in a place); stub 17 writes a value in 0..4095 through its argument 0, printed after the answers (`o` in a place); the call ends where stub 19 is first entered (the function is swept to that callee); the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p22+0` as well as at `this`; the stubbed vtable laid at `p23+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_come_out_edx.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (line, calls) = line.split_once(" # ").expect("`<row> # <calls>`");
        let want_log: Vec<&str> = calls.split_whitespace().collect();
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(4);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
            // Window layout (sweep_gen's flag order): `m` the unit; `c[0]` the objects record (`0xc0618c`), `c[1]` the
            // constants, `c[2]` the terrain cell, `c[3]` the leader record `leaders[3]` (`0xe4eff4`). `p[1+h]` the
            // host objects of player 3, `p[6+h]` their `BuildData`, `p[11+h]` the units array's records, `p[16+i]`
            // the unit's figures, `p[19]` its type, `p[20+h]` / `p[24+h]` the objects' and units' types,
            // `p[28+h]` the gather node, `p[32]` the `div_3_table`. The answers `a`: `a[0]` the argument,
            // `a[1+stub]` the stubs' single answers, `a[21..25]` the out-words (get_inside's player, find_nearby_spot's
            // two, find_tcoord_z's), `a[25..52]` the call-order answers by `--seq`, `a[52]`, `a[53]` the two image flags.
            use crate::come_out_view::{ComeOutView, Exit, Find, Found, Node, come_out_steps};
            const ENC: i32 = 0x63637;
            struct V<'a> {
                m: &'a mut [u8],
                c: &'a mut [Vec<u8>],
                p: &'a mut [Vec<u8>],
                a: &'a [i32],
                log: &'a mut Vec<String>,
                angles: Vec<String>,
                cur: [usize; 14],
            }
            fn r32(w: &[u8], o: usize) -> i32 {
                i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
            }
            fn w32(w: &mut [u8], o: usize, v: i32) {
                w[o..o + 4].copy_from_slice(&v.to_le_bytes());
            }
            impl V<'_> {
                /// The next call-order answer of stub `s` (`--seq`), the last repeated.
                fn seq(&mut self, s: usize) -> i32 {
                    let (at, len) = match s {
                        0 => (25, 3),
                        1 => (28, 2),
                        2 => (30, 2),
                        3 => (32, 3),
                        6 => (35, 4),
                        7 => (39, 4),
                        9 => (43, 1),
                        10 => (44, 6),
                        12 => (50, 1),
                        13 => (51, 1),
                        _ => unreachable!(),
                    };
                    let k = self.cur[s].min(len - 1);
                    self.cur[s] += 1;
                    self.a[at + k]
                }
                fn idx(&self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                }
                fn rec(&mut self, _s: String) {}
                fn obj(&self, player: i32, h: i32) -> usize {
                    assert_eq!(player, 3, "only player 3's arrays are laid");
                    assert!((0..4).contains(&h), "host {h}");
                    1 + h as usize
                }
            }
            impl ComeOutView for V<'_> {
                fn group_clear(&mut self) {}
                fn member_count(&self) -> i32 {
                    i32::from(self.m[0xb5] as i8)
                }
                fn clear_member(&mut self, i: i32) {
                    w32(&mut self.p[16 + i as usize], 0xd0, 0);
                }
                fn is_captain(&mut self) -> bool {
                    if self.a[52] != 0 {
                        return (u16::from_le_bytes([self.m[0x8e], self.m[0x8f]]) >> 15) != 0;
                    }
                    self.rec("s0:m+0x0".into());
                    self.seq(0) != 0
                }
                fn get_inside(&mut self) -> (i32, i32) {
                    self.rec("s9:m+0x0,sp-0xa20".into());
                    (self.seq(9), self.a[21])
                }
                fn pos(&self) -> (i32, i32) {
                    (r32(self.m, 0x10) ^ ENC, r32(self.m, 0x14) ^ ENC)
                }
                fn owner(&self) -> i32 {
                    i32::from(self.m[9])
                }
                fn block_radius(&self) -> i32 {
                    r32(&self.p[19], 0x240)
                }
                fn domain(&self) -> i32 {
                    r32(&self.p[19], 0x218)
                }
                fn big_radius(&self) -> i32 {
                    r32(&self.p[19], 0x244)
                }
                fn missile(&self) -> bool {
                    r32(&self.p[19], 0x1e4) & 0x800_0000 != 0
                }
                fn disembark(&self) -> i32 {
                    r32(&self.c[1], 0x9c)
                }
                fn train(&self) -> (i32, i32) {
                    (r32(&self.c[1], 0x84), r32(&self.c[1], 0x88))
                }
                fn boat_train(&self) -> (i32, i32) {
                    (r32(&self.c[1], 0x8c), r32(&self.c[1], 0x90))
                }
                fn find(&mut self, f: Find) -> Found {
                    let (o1, o2) = if f.gather { ("sp-0xa0c", "sp-0xa10") } else { ("sp-0xa18", "sp-0xa14") };
                    let (idx, owner) = (self.idx(), self.owner());
                    self.rec(format!(
                        "s10:p19+0x0,{},{},{o1},{o2},{},{},0,{},{},{idx},{owner},{},{},-1,0,-1",
                        f.x, f.y, f.min, f.max, f.angle, f.filter, f.a12, f.a13
                    ));
                    Found { code: self.seq(10), x: self.a[22], y: self.a[23] }
                }
                fn host_is(&mut self, player: i32, h: i32, id: i32) -> bool {
                    let o = self.obj(player, h);
                    if self.a[53] != 0 {
                        self.rec(format!("s7:p{}+0x0,{id},0", 20 + h));
                        self.seq(7) != 0
                    } else {
                        self.rec(format!("s6:p{o}+0x0,{id},0"));
                        self.seq(6) != 0
                    }
                }
                fn barge(&mut self, player: i32, h: i32) -> i32 {
                    let owner = self.owner();
                    assert_eq!(owner, 3, "only leaders[3] is laid");
                    self.rec("s11:c3+0x0,320".into());
                    let up = self.a[12];
                    let ty = if up < 0 { 0x140 } else { up };
                    let (x, y) = self.object_pos(player, h);
                    self.rec(format!("s12:c0+0x0,{owner},{ty},{x},{y},-1,-1,-1"));
                    self.seq(12)
                }
                fn object_pos(&self, player: i32, h: i32) -> (i32, i32) {
                    let o = self.obj(player, h);
                    (r32(&self.p[o], 0x10) ^ ENC, r32(&self.p[o], 0x14) ^ ENC)
                }
                fn set_figure_90(&mut self) {
                    w32(&mut self.p[16], 0x40, 0x42b4_0000);
                }
                fn v20(&mut self, player: i32, h: i32) -> bool {
                    let o = self.obj(player, h);
                    self.rec(format!("s3:p{o}+0x0"));
                    self.seq(3) != 0
                }
                fn v34(&mut self, player: i32, h: i32) {
                    let o = self.obj(player, h);
                    self.rec(format!("s4:p{o}+0x0"));
                }
                fn v1c(&mut self, player: i32, h: i32) -> bool {
                    let o = self.obj(player, h);
                    self.rec(format!("s2:p{o}+0x0"));
                    self.seq(2) != 0
                }
                fn get_captain(&mut self) -> i32 {
                    self.rec("s1:m+0x0".into());
                    self.seq(1)
                }
                fn set_leader_flag(&mut self) {
                    let v = r32(&self.c[3], 0) | 0x200_0000;
                    w32(&mut self.c[3], 0, v);
                }
                fn build_data(&mut self, player: i32, h: i32) -> i32 {
                    let o = self.obj(player, h);
                    self.rec(format!("s5:p{o}+0x0"));
                    h
                }
                fn bd_has_list(&self, bd: i32) -> bool {
                    r32(&self.p[6 + bd as usize], 0xcc) != 0
                }
                fn bd_gather_inside(&mut self, bd: i32) -> bool {
                    self.rec(format!("s13:p{}+0x0", 6 + bd));
                    self.seq(13) != 0
                }
                fn bd_head(&mut self, bd: i32) {
                    self.rec(format!("s14:p{}+0xbc", 6 + bd));
                }
                fn bd_node(&self, bd: i32) -> Node {
                    let n = &self.p[28 + bd as usize];
                    Node { kind: i32::from(n[0xc]), a: r32(n, 4), b: r32(n, 8) }
                }
                fn copy_hotkeys(&mut self, a: i32, b: i32) {
                    let (idx, owner) = (self.idx(), self.owner());
                    self.rec(format!("s15:{a},{b},{idx},{owner}"));
                }
                fn find_angle(&mut self, dx: i32, dy: i32) -> i32 {
                    self.angles.push(format!("{dx},{dy}"));
                    self.a[17]
                }
                fn set_unit_angle(&mut self, a: i32) {
                    let i = 11 + self.idx() as usize;
                    w32(&mut self.p[i], 0x50, a);
                }
                fn object_alive(&self, player: i32, h: i32) -> bool {
                    r32(&self.p[self.obj(player, h)], 8) & 1 != 0
                }
                fn object_size(&self, player: i32, h: i32) -> (i32, i32) {
                    let t = &self.p[19 + self.obj(player, h)];
                    (r32(t, 0x234), r32(t, 0x238))
                }
                fn host_angle(&self, player: i32, h: i32) -> i32 {
                    r32(&self.p[10 + self.obj(player, h)], 0x50)
                }
                fn host_block_radius(&self, player: i32, h: i32) -> i32 {
                    r32(&self.p[23 + self.obj(player, h)], 0x240)
                }
                fn set_pos(&mut self, x: i32, y: i32) {
                    w32(self.m, 0x10, x ^ ENC);
                    w32(self.m, 0x14, y ^ ENC);
                }
                fn find_tcoord_z(&mut self, x: i32, y: i32) {
                    let tx = r32(&self.p[32], 4 * (x >> 6) as usize);
                    let ty = r32(&self.p[32], 4 * (y >> 6) as usize);
                    self.rec(format!("s17:c2+0x0,sp-0xa40,{tx},{ty},1"));
                    w32(self.m, 0xc, self.a[24] ^ ENC);
                }
                fn remove_from_inside(&mut self) {
                    self.rec("s18:m+0x0".into());
                }
            }
            let mut v = V { m, c, p, a, log: &mut log, angles: Vec::new(), cur: [0; 14] };
            let exit = come_out_steps(&mut v, a[0]);
            match exit {
                Exit::Place { .. } | Exit::Ret(_) => {}
                other => panic!("an arm the sweep does not reach: {other:?}, row {}", &line[..line.len().min(150)]),
            }
            let col = if v.angles.is_empty() { "-".to_string() } else { v.angles.join("/") };
            v.log.push(col);
            0
}
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "come_out — the recorded calls, row {}", &line[..line.len().min(150)]);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "come_out — this's bytes, row {}", &line[..line.len().min(150)]);
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "come_out — window c[{i}], row {}", &line[..line.len().min(150)]);
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[5 + i], "come_out — window p[{i}], row {}", &line[..line.len().min(150)]);
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Unit::come_out@00617c10: {n} rows agree");
}
