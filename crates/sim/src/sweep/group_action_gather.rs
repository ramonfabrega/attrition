//! `Group::action_gather@00700b90` under the emulator against the port (item 3386; tools/sweep_gen.py, obj with windows).

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

/// `Group::action_gather@00700b90`, run by `tools/emu/sweep_group_action_gather.py` under unicorn
/// (tools/emu): 2272 random bytes of object, 2 argument(s), `p` windows in order: p0..p109 the place(s), in flag order, p110..p113 the stubs' answer window(s); the global cell 0x00c0618c points at 64 random bytes; the global cell 0x00cae5fc points at 256 random bytes; the global cell 0x00c06188 points at 320 random bytes; the global cell 0x00c061a8 points at 32 random bytes; the global cell 0x00c061ec points at 2096 random bytes; 4 random bytes at 0x00cc02f8 (each word in 0..0); 4 random bytes at 0x00e3a390 (each word in 0..8191); 4 random bytes at 0x00e4127c (each word in 0..8191); vtable slot +0x14 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x60 (2 args) stubbed; vtable slot +0xac (0 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0xc (0 args) stubbed; vtable slot +0x4c (0 args) stubbed; vtable slot +0xb0 (0 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x158 (0 args) stubbed; callee 0x00713e80 (1 args) stubbed; callee 0x00711720 (3 args) stubbed; callee 0x006e33a0 (2 args) stubbed; callee 0x00714350 (4 args) stubbed; callee 0x00675bc0 (3 args) stubbed; callee 0x0064dda0 (2 args) stubbed; callee 0x00608450 (0 args) stubbed; callee 0x006b52e0 (2 args) stubbed; callee 0x005e4a60 (7 args) stubbed; callee 0x0061a5c0 (3 args) stubbed; callee 0x00646c40 (1 args) stubbed; callee 0x005f9400 (2 args) stubbed; callee 0x005e3860 (0 args) stubbed; callee 0x0061a2e0 (3 args) stubbed; callee 0x00647080 (3 args) stubbed; callee 0x0062f710 (0 args) stubbed; callee 0x00714700 (0 args) stubbed; callee 0x00704990 (11 args) stubbed; the word at `c0+0x14+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 96 random bytes less `0` (placed); the word at `p0+4` points at 96 random bytes less `0` (placed); the word at `p0+8` points at 96 random bytes less `0` (placed); the word at `p0+12` points at 96 random bytes less `0` (placed); the word at `c0+0x14+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 96 random bytes less `0` (placed); the word at `p5+4` points at 96 random bytes less `0` (placed); the word at `p5+8` points at 96 random bytes less `0` (placed); the word at `p5+12` points at 96 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*0` points at 16 random bytes less `0` (placed); the word at `p10+0` points at 144 random bytes less `0` (placed); the word at `p10+4` points at 144 random bytes less `0` (placed); the word at `p10+8` points at 144 random bytes less `0` (placed); the word at `p10+12` points at 144 random bytes less `0` (placed); the word at `0xc0aec0+0x1c*1` points at 16 random bytes less `0` (placed); the word at `p15+0` points at 144 random bytes less `0` (placed); the word at `p15+4` points at 144 random bytes less `0` (placed); the word at `p15+8` points at 144 random bytes less `0` (placed); the word at `p15+12` points at 144 random bytes less `0` (placed); the word at `0xe85ddc` points at 216 random bytes less `0` (placed); the word at `p20+0` points at 8 random bytes less `0` (placed); the word at `p20+4` points at 8 random bytes less `0` (placed); the word at `p20+8` points at 8 random bytes less `0` (placed); the word at `p20+12` points at 8 random bytes less `0` (placed); the word at `p20+16` points at 8 random bytes less `0` (placed); the word at `p20+20` points at 8 random bytes less `0` (placed); the word at `p20+24` points at 8 random bytes less `0` (placed); the word at `p20+28` points at 8 random bytes less `0` (placed); the word at `p20+32` points at 8 random bytes less `0` (placed); the word at `p20+36` points at 8 random bytes less `0` (placed); the word at `p20+40` points at 8 random bytes less `0` (placed); the word at `p20+44` points at 8 random bytes less `0` (placed); the word at `p20+48` points at 8 random bytes less `0` (placed); the word at `p20+52` points at 8 random bytes less `0` (placed); the word at `p20+56` points at 8 random bytes less `0` (placed); the word at `p20+60` points at 8 random bytes less `0` (placed); the word at `p20+64` points at 8 random bytes less `0` (placed); the word at `p20+68` points at 8 random bytes less `0` (placed); the word at `p20+72` points at 8 random bytes less `0` (placed); the word at `p20+76` points at 8 random bytes less `0` (placed); the word at `p20+80` points at 8 random bytes less `0` (placed); the word at `p20+84` points at 8 random bytes less `0` (placed); the word at `p20+88` points at 8 random bytes less `0` (placed); the word at `p20+92` points at 8 random bytes less `0` (placed); the word at `p20+96` points at 8 random bytes less `0` (placed); the word at `p20+100` points at 8 random bytes less `0` (placed); the word at `p20+104` points at 8 random bytes less `0` (placed); the word at `p20+108` points at 8 random bytes less `0` (placed); the word at `p20+112` points at 8 random bytes less `0` (placed); the word at `p20+116` points at 8 random bytes less `0` (placed); the word at `p20+120` points at 8 random bytes less `0` (placed); the word at `p20+124` points at 8 random bytes less `0` (placed); the word at `p20+128` points at 8 random bytes less `0` (placed); the word at `p20+132` points at 8 random bytes less `0` (placed); the word at `p20+136` points at 8 random bytes less `0` (placed); the word at `p20+140` points at 8 random bytes less `0` (placed); the word at `p20+144` points at 8 random bytes less `0` (placed); the word at `p20+148` points at 8 random bytes less `0` (placed); the word at `p20+152` points at 8 random bytes less `0` (placed); the word at `p20+156` points at 8 random bytes less `0` (placed); the word at `p20+160` points at 8 random bytes less `0` (placed); the word at `p20+164` points at 8 random bytes less `0` (placed); the word at `p20+168` points at 8 random bytes less `0` (placed); the word at `p20+172` points at 8 random bytes less `0` (placed); the word at `p20+176` points at 8 random bytes less `0` (placed); the word at `p20+180` points at 8 random bytes less `0` (placed); the word at `p20+184` points at 8 random bytes less `0` (placed); the word at `p20+188` points at 8 random bytes less `0` (placed); the word at `p20+192` points at 8 random bytes less `0` (placed); the word at `p20+196` points at 8 random bytes less `0` (placed); the word at `p20+200` points at 8 random bytes less `0` (placed); the word at `p20+204` points at 8 random bytes less `0` (placed); the word at `p20+208` points at 8 random bytes less `0` (placed); the word at `p20+212` points at 8 random bytes less `0` (placed); the word at `p1+0x18` points at 32 random bytes less `0` (placed); the word at `p2+0x18` points at 32 random bytes less `0` (placed); the word at `p3+0x18` points at 32 random bytes less `0` (placed); the word at `p4+0x18` points at 32 random bytes less `0` (placed); the word at `p6+0x18` points at 32 random bytes less `0` (placed); the word at `p7+0x18` points at 32 random bytes less `0` (placed); the word at `p8+0x18` points at 32 random bytes less `0` (placed); the word at `p9+0x18` points at 32 random bytes less `0` (placed); the word at `p1+0x40` points at 32 random bytes less `0` (placed); the word at `p2+0x40` points at 32 random bytes less `0` (placed); the word at `p3+0x40` points at 32 random bytes less `0` (placed); the word at `p4+0x40` points at 32 random bytes less `0` (placed); the word at `p6+0x40` points at 32 random bytes less `0` (placed); the word at `p7+0x40` points at 32 random bytes less `0` (placed); the word at `p8+0x40` points at 32 random bytes less `0` (placed); the word at `p9+0x40` points at 32 random bytes less `0` (placed); the word at `p83+0x18` points at 32 random bytes less `0` (placed); the word at `p84+0x18` points at 32 random bytes less `0` (placed); the word at `p85+0x18` points at 32 random bytes less `0` (placed); the word at `p86+0x18` points at 32 random bytes less `0` (placed); the word at `p87+0x18` points at 32 random bytes less `0` (placed); the word at `p88+0x18` points at 32 random bytes less `0` (placed); the word at `p89+0x18` points at 32 random bytes less `0` (placed); the word at `p90+0x18` points at 32 random bytes less `0` (placed); the word at `p11+0x18` points at 32 random bytes less `0` (placed); the word at `p12+0x18` points at 32 random bytes less `0` (placed); the word at `p13+0x18` points at 32 random bytes less `0` (placed); the word at `p14+0x18` points at 32 random bytes less `0` (placed); the word at `p16+0x18` points at 32 random bytes less `0` (placed); the word at `p17+0x18` points at 32 random bytes less `0` (placed); the word at `p18+0x18` points at 32 random bytes less `0` (placed); the word at `p19+0x18` points at 32 random bytes less `0` (placed); the word at `c3+0x10` points at 2656 random bytes less `0` (placed); the word at `c2+0x134` points at 512 random bytes less `0` (placed); the word at `c2+0x138` points at 512 random bytes less `0` (placed); slot +0xb8 laid as 0x00653790 or its stub, a flag after the answers; slot +0x158 laid as 0x0060eda0 or its stub, a flag after the answers; stub 1 answered from [7, 1, 3, 0, 2] (90%); stub 2 answered from [0, 1, 1, 1] (90%); stub 3 answered from [0, 1, 1, 1] (90%); stub 5 answered from [1, 1, 1, 1, 0] (90%); stub 6 answered from [1, 1, 1, 1, 0] (90%); stub 7 answered from [1, 1, 1, 0] (90%); stub 8 answered from [1, 1, 1, 0] (90%); stub 10 answered from [0, 1, 1, 1] (90%); stub 13 answered from [0, 0, 1, 2] (90%); stub 14 answered from [1, 1, 1, 1, 0] (90%); stub 16 answered from [0, 0, 0, 1] (90%); stub 17 answered from [0, 1, 1] (90%); stub 22 answered from [0, 1, 1] (90%); stub 28 answered from [0, 1, 1, 2] (90%); stub 18 answers a window of its own to each call, in call order (-64 bytes, ...); stub 1 answers 12 values in call order, printed after the answers; stub 2 answers 12 values in call order, printed after the answers; stub 3 answers 12 values in call order, printed after the answers; stub 5 answers 24 values in call order, printed after the answers; stub 6 answers 24 values in call order, printed after the answers; stub 7 answers 24 values in call order, printed after the answers; stub 8 answers 24 values in call order, printed after the answers; stub 10 answers 24 values in call order, printed after the answers; stub 13 answers 3 values in call order, printed after the answers; stub 14 answers 2 values in call order, printed after the answers; stub 16 answers 16 values in call order, printed after the answers; stub 17 answers 8 values in call order, printed after the answers; stub 22 answers 4 values in call order, printed after the answers; stub 28 answers 2 values in call order, printed after the answers; stub 19 answers 40 values in call order, printed after the answers; stub 19 answers 0..3 every row; stub 0 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 1 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 2 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 3 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 9 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 10 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 11 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 12 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 13 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 14 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 15 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 16 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 17 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 18 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 19 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 20 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 21 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 22 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 23 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 24 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 25 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 26 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 27 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 28 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 29 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the recorded calls one token a call, in call order, each headed `sIDX:` (the callee); stub 19 prints the 4 bytes its argument 0 points at, hex, after the record (`/` a call); stub 19 prints the 4 bytes its argument 1 points at, hex, after the record (`/` a call); the stack 2816 bytes under the call is zeroed before each row; stub 4 answers from its own `ecx`: the word at `+0x40`; stub 9 answers from its own `ecx`: the word at `+0x40`; figures p1..p4 link at +0x2c (2 bytes) below their own index or to -1; figures p6..p9 link at +0x2c (2 bytes) below their own index or to -1; figures p1..p4 hold a short at +0x2e drawn from [0, 1], after the links; figures p6..p9 hold a short at +0x2e drawn from [0, 1], after the links; the stubbed vtable laid at `p75+0` as well as at `this`; the stubbed vtable laid at `p76+0` as well as at `this`; the stubbed vtable laid at `p77+0` as well as at `this`; the stubbed vtable laid at `p78+0` as well as at `this`; the stubbed vtable laid at `p79+0` as well as at `this`; the stubbed vtable laid at `p80+0` as well as at `this`; the stubbed vtable laid at `p81+0` as well as at `this`; the stubbed vtable laid at `p82+0` as well as at `this`; the stubbed vtable laid at `p91+0` as well as at `this`; the stubbed vtable laid at `p92+0` as well as at `this`; the stubbed vtable laid at `p93+0` as well as at `this`; the stubbed vtable laid at `p94+0` as well as at `this`; the stubbed vtable laid at `p95+0` as well as at `this`; the stubbed vtable laid at `p96+0` as well as at `this`; the stubbed vtable laid at `p97+0` as well as at `this`; the stubbed vtable laid at `p98+0` as well as at `this`; the stubbed vtable laid at `p110+0` as well as at `this`; the stubbed vtable laid at `p111+0` as well as at `this`; the stubbed vtable laid at `p112+0` as well as at `this`; the stubbed vtable laid at `p113+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; the stubbed vtable laid at `p14+0` as well as at `this`; the stubbed vtable laid at `p16+0` as well as at `this`; the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 2000 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_action_gather.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(8);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // Window layout. `m` the group (`+0xc` count, `+0x10` form, `+0x4a` player, the shorts at
                // `+0x8cc`); `c[0]` the objects cell, `c[1]` the `div_3_table`, `c[2]` the world, `c[3]` the
                // spell types cell, `c[4]` the game, `c[5]` `ignore_orders` (held at 0: the head's list is
                // `group_action_gather_head`'s), `c[6]`/`c[7]` the first word of players 0 and 1's leaders.
                // `p[0..=4]` the objects' array of player 0 and its four objects, `p[5..=9]` player 1's,
                // `p[10..=14]` and `p[15..=19]` the units', `p[20]` the types' array and `p[21..=74]` its
                // records, `p[75 + 4 * pl + o]` an object's own type, `p[83 + ..]` its data (`+0x40`),
                // `p[91 + ..]` that data's type, `p[99 + ..]` a unit's type, `p[107]` the spell types,
                // `p[108]` the tile cells, `p[109]` the world's second array, `p[110..=113]` the orders
                // `get_action` answers. `a`: the two arguments, the stubs' answers (`a[2 + k]`, stub `k`),
                // every `--seq` run (`SEQ_AT`), the two `--slotbase` flags and the four null flags.
                //
                // **The port is the crate's own steps**, `crate::action_gather_view::action_gather_steps`,
                // over a view of the emulated original's windows; the crate has no `Group::action_gather`
                // to call (see that module's head).
                use crate::action_gather_view::{GarrisonHead, GatherView, action_gather_steps};
                const SEQ_AT: [(usize, usize, usize); 15] = [
                    (1, 32, 12),
                    (2, 44, 12),
                    (3, 56, 12),
                    (5, 68, 24),
                    (6, 92, 24),
                    (7, 116, 24),
                    (8, 140, 24),
                    (10, 164, 24),
                    (13, 188, 3),
                    (14, 191, 2),
                    (16, 193, 16),
                    (17, 209, 8),
                    (22, 217, 4),
                    (28, 221, 2),
                    (19, 223, 40),
                ];
                const FLAG_B8: usize = 263;
                const FLAG_158: usize = 264;
                const NULL_AT: usize = 265;
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a mut [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    log: &'a mut Vec<String>,
                    turn: [usize; 30],
                    deref: [Vec<String>; 2],
                }
                fn rd32(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                fn rd16(w: &[u8], o: usize) -> i32 {
                    i32::from(i16::from_le_bytes([w[o], w[o + 1]]))
                }
                fn ow(p: i32, o: i32) -> usize {
                    (if p == 0 { 1 } else { 6 }) + o as usize
                }
                fn uw(p: i32, u: i32) -> usize {
                    (if p == 0 { 11 } else { 16 }) + u as usize
                }
                fn k(p: i32, o: i32) -> usize {
                    4 * p as usize + o as usize
                }
                impl V<'_> {
                    fn seq(&mut self, s: usize) -> i32 {
                        let &(_, at, n) = SEQ_AT.iter().find(|e| e.0 == s).expect("a seq stub");
                        let i = self.turn[s].min(n - 1);
                        self.turn[s] += 1;
                        self.a[at + i]
                    }
                    fn rec(&mut self, s: usize, ecx: &str, args: &[i32]) {
                        let mut t = format!("s{s}:{ecx}");
                        for v in args {
                            t.push_str(&format!(",{v}"));
                        }
                        self.log.push(t);
                    }
                }
                impl GarrisonHead for V<'_> {
                    fn who(&self) -> i32 {
                        i32::from(self.m[0x4a])
                    }
                    fn ignore_orders(&self) -> bool {
                        rd32(&self.c[5], 0) != 0
                    }
                    fn ignoring_count(&self, _who: i32) -> i32 {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn ignoring_at(&self, _who: i32, _i: i32) -> i32 {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn drop_ignored(&mut self, _o: i32, _who: i32) {
                        unreachable!("the whole-function sweep keeps ignore_orders at 0")
                    }
                    fn refresh(&mut self) {
                        self.rec(0, "m+0x0", &[]);
                    }
                }
                impl GatherView for V<'_> {
                    fn list_clear(&mut self) {
                        self.rec(12, "sp-0xa20", &[-1]);
                    }
                    fn set_form(&mut self) {
                        self.m[0x10..0x14].copy_from_slice(&(-1i32).to_le_bytes());
                    }
                    fn count(&self) -> i32 {
                        rd32(self.m, 0xc)
                    }
                    fn member(&self, i: i32) -> i32 {
                        rd16(self.m, 0x8cc + 2 * i as usize)
                    }
                    fn count_kind(&mut self, kind: i32, ty: i32) -> i32 {
                        self.rec(13, "m+0x0", &[kind, ty, 0]);
                        self.seq(13)
                    }
                    fn world_w(&self) -> i32 {
                        rd32(&self.c[2], 0)
                    }
                    fn world_dim_x(&self) -> i32 {
                        rd32(&self.c[2], 0x18)
                    }
                    fn world_dim_y(&self) -> i32 {
                        rd32(&self.c[2], 0x1c)
                    }
                    fn div3(&self, i: i32) -> i32 {
                        rd32(&self.c[1], 4 * i as usize)
                    }
                    fn cell_i16(&self, idx: i32, off: i32) -> i32 {
                        rd16(&self.p[108], 0x1c * idx as usize + off as usize)
                    }
                    fn ext_byte(&self, i: i32) -> i32 {
                        i32::from(self.p[109][2 * i as usize])
                    }
                    fn get_tregion(&mut self, tx: i32, ty: i32) -> i32 {
                        let hex = |v: i32| v.to_le_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>();
                        self.log.push("s19:c2+0x0,sp-0x18,sp-0x14".into());
                        self.deref[0].push(hex(tx));
                        self.deref[1].push(hex(ty));
                        self.seq(19)
                    }
                    fn obj_pos_raw(&self, p: i32, o: i32) -> (i32, i32) {
                        let w = &self.p[ow(p, o)];
                        (rd32(w, 0x10), rd32(w, 0x14))
                    }
                    fn obj_alive(&mut self, p: i32, o: i32) -> bool {
                        self.rec(5, &format!("p{}+0x0", ow(p, o)), &[]);
                        self.seq(5) != 0
                    }
                    fn obj_bc(&mut self, p: i32, o: i32) -> bool {
                        self.rec(6, &format!("p{}+0x0", ow(p, o)), &[]);
                        self.seq(6) != 0
                    }
                    fn obj_is(&mut self, p: i32, o: i32, ty: i32) -> bool {
                        if self.a[FLAG_B8] != 0 {
                            self.rec(3, &format!("p{}+0x0", 75 + k(p, o)), &[ty, 0]);
                            self.seq(3) != 0
                        } else {
                            self.rec(2, &format!("p{}+0x0", ow(p, o)), &[ty, 0]);
                            self.seq(2) != 0
                        }
                    }
                    fn obj_data_kind(&mut self, p: i32, o: i32) -> i32 {
                        self.rec(4, &format!("p{}+0x0", ow(p, o)), &[]);
                        rd32(&self.p[91 + k(p, o)], 4)
                    }
                    fn type_avail(&mut self, p: i32, kind: i32) -> bool {
                        self.rec(14, &format!("c{}+0x0", 6 + p), &[kind, 1]);
                        self.seq(14) != 0
                    }
                    fn leader_flag(&self, p: i32) -> bool {
                        rd32(&self.c[6 + p as usize], 0) & 0x700 != 0
                    }
                    fn unit_type_override(&self, p: i32, u: i32) -> i32 {
                        rd32(&self.p[uw(p, u)], 0x54)
                    }
                    fn unit_kind(&self, p: i32, u: i32) -> i32 {
                        rd32(&self.p[99 + k(p, u)], 4)
                    }
                    fn type_kind(&self, ty: i32) -> i32 {
                        rd32(&self.p[21 + ty as usize], 4)
                    }
                    fn is_castable(&mut self, p: i32, u: i32) -> bool {
                        let ecx = rd32(&self.p[107], 0xa4c);
                        self.log.push(format!("s16:{ecx},{u},{p},0"));
                        self.seq(16) != 0
                    }
                    fn count_inside(&mut self, p: i32, u: i32, kind: i32, ty: i32) -> i32 {
                        self.rec(17, &format!("p{}+0x0", uw(p, u)), &[kind, ty]);
                        self.seq(17)
                    }
                    fn add_local(&mut self, u: i32, p: i32) {
                        self.rec(15, "sp-0xa20", &[u, p, 0, 0]);
                    }
                    fn unit_order_kind(&mut self, p: i32, u: i32) -> Option<i32> {
                        self.rec(18, &format!("p{}+0x0", uw(p, u)), &[]);
                        let n = self.turn[18].min(3);
                        self.turn[18] += 1;
                        if self.a[NULL_AT + n] != 0 {
                            return None;
                        }
                        self.rec(1, &format!("p{}+0x0", 110 + n), &[]);
                        Some(self.seq(1))
                    }
                    fn obj_next(&self, p: i32, o: i32) -> (i32, i32) {
                        let w = &self.p[ow(p, o)];
                        (rd16(w, 0x2c), rd16(w, 0x2e))
                    }
                    fn obj_slot_c(&mut self, p: i32, o: i32) -> bool {
                        self.rec(7, &format!("p{}+0x0", ow(p, o)), &[]);
                        self.seq(7) != 0
                    }
                    fn obj_slot_4c(&mut self, p: i32, o: i32) -> bool {
                        self.rec(8, &format!("p{}+0x0", ow(p, o)), &[]);
                        self.seq(8) != 0
                    }
                    fn obj_data_slot_90(&mut self, p: i32, o: i32) -> bool {
                        self.rec(9, &format!("p{}+0x0", ow(p, o)), &[]);
                        self.rec(10, &format!("p{}+0x0", 91 + k(p, o)), &[]);
                        self.seq(10) != 0
                    }
                    fn unit_clear_80(&mut self, p: i32, u: i32) {
                        self.p[uw(p, u)][0x80..0x82].copy_from_slice(&[0xff, 0xff]);
                    }
                    fn add_cast_order(&mut self, p: i32, u: i32, x: i32, y: i32, queue: i32) {
                        self.rec(20, &format!("p{}+0x0", uw(p, u)), &[-1, -1, x, y, 0x293, queue, 1]);
                    }
                    fn add_gather_order(&mut self, p: i32, u: i32, target: i32, queue: i32) {
                        self.rec(21, &format!("p{}+0x0", uw(p, u)), &[target, queue, 1]);
                    }
                    fn semaphore(&self) -> bool {
                        self.c[4][0x821] & 8 != 0
                    }
                    fn obj_link28(&self, p: i32, o: i32) -> i32 {
                        rd16(&self.p[ow(p, o)], 0x28)
                    }
                    fn can_carry_ground(&mut self, p: i32, u: i32) -> bool {
                        self.rec(22, &format!("p{}+0x0", uw(p, u)), &[0]);
                        self.seq(22) != 0
                    }
                    fn unit_carrier(&self, p: i32, u: i32) -> (i32, i32) {
                        let w = &self.p[uw(p, u)];
                        (i32::from(w[0x3e] as i8), rd16(w, 0x28))
                    }
                    fn same_damage(&mut self, owner: i32, cont: i32, u: i32, p: i32) {
                        self.rec(23, &format!("p{}+0x0", uw(owner, cont)), &[u, p]);
                    }
                    fn clear_orders(&mut self, p: i32, u: i32) {
                        self.rec(24, &format!("p{}+0x0", uw(p, u)), &[]);
                    }
                    fn go_inside(&mut self, p: i32, u: i32, target: i32, tp: i32) {
                        self.rec(25, &format!("p{}+0x0", uw(p, u)), &[target, tp, 0]);
                    }
                    fn die(&mut self, p: i32, u: i32) {
                        if self.a[FLAG_158] != 0 {
                            self.rec(26, &format!("p{}+0x0", uw(p, u)), &[0, -1, 0]);
                        } else {
                            self.rec(11, &format!("p{}+0x0", uw(p, u)), &[]);
                        }
                    }
                    fn check_gatherers(&mut self, p: i32, o: i32) {
                        self.rec(4, &format!("p{}+0x0", ow(p, o)), &[]);
                        self.rec(27, &format!("p{}+0x0", 83 + k(p, o)), &[]);
                    }
                    fn group_slot10(&mut self, _u: i32, _p: i32) {
                        self.rec(1, "m+0x0", &[]);
                        self.seq(1);
                    }
                    fn get_num(&mut self) -> i32 {
                        self.rec(28, "sp-0xa20", &[]);
                        self.seq(28)
                    }
                    fn move_near(&mut self, x: i32, y: i32) {
                        self.rec(29, "sp-0xa20", &[x, y, 192, 2, 0, 0, 1, 0, -1, -1, 0]);
                    }
                }
                let mut v = V {
                    a,
                    m: &mut *m,
                    c: &*c,
                    p: &mut *p,
                    log: &mut log,
                    turn: [0; 30],
                    deref: [Vec::new(), Vec::new()],
                };
                action_gather_steps(&mut v, a[0], a[1]);
                let [d0, d1] = std::mem::take(&mut v.deref);
                let tail = |d: Vec<String>| {
                    if d.is_empty() {
                        "-".to_string()
                    } else {
                        d.join("/")
                    }
                };
                v.log.push(tail(d0));
                v.log.push(tail(d1));
                0
            }
        };
        let _ = (got, want_eax);
        assert_eq!(log, want_log, "action_gather — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "action_gather — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "action_gather — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[9 + i], "action_gather — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 2000, "only {n} rows");
    eprintln!("Group::action_gather@00700b90: {n} rows agree");
}
