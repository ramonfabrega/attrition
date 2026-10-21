//! `Army::is_moving@006f5470` under the emulator against the port (item 3395; tools/sweep_gen.py, obj with windows).

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

/// `Army::is_moving@006f5470`, run by `tools/emu/sweep_army_is_moving.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p23 the place(s), in flag order; the global cell 0x00c06188 points at 312 random bytes; the global cell 0x00c0618c points at 64 random bytes; callee 0x006f9df0 (1 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; vtable slot +0x14 (0 args) stubbed; callee 0x006f8a20 (2 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 256 random bytes less `0` (placed); the word at `p0+4` points at 256 random bytes less `0` (placed); the word at `p0+8` points at 256 random bytes less `0` (placed); the word at `p0+12` points at 256 random bytes less `0` (placed); the word at `c1+0x14` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p5+4` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); the word at `p5+12` points at 32 random bytes less `0` (placed); the word at `p1+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p2+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p3+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p4+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p10+4` points at 16 random bytes less `0` (placed); the word at `p11+4` points at 16 random bytes less `0` (placed); the word at `p12+4` points at 16 random bytes less `0` (placed); the word at `p13+4` points at 16 random bytes less `0` (placed); the word at `p14+8` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p15+8` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p16+8` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `p17+8` points at 32 random bytes less `0` (placed, null a third of the rows); the word at `c0+0x134` points at 12288 random bytes less `0` (placed); the word at `0xcae5fc` points at 1024 random bytes less `0` (placed); stub 1 answered from [1, 1, 1, 0] (90%); stub 2 answered from [1, 1, 1, 0] (90%); stub 3 answered from [0, 1, 1] (90%); stub 0 answers 12 values in call order, printed after the answers; stub 1 answers 12 values in call order, printed after the answers; stub 2 answers 12 values in call order, printed after the answers; stub 3 answers 12 values in call order, printed after the answers; stub 0 answers -1..3 every row; stub 4 writes a value in 0..4 through its argument 0, printed after the answers (`o` in a place); stub 4 writes a value in 0..4 through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; the stubbed vtable laid at `p20+0` as well as at `this`; the stubbed vtable laid at `p21+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p7+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_is_moving.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
        let (lhs, rhs) = line.split_once(" -> ").expect("a row is `<args…> | <windows> -> <eax> | <windows>`");
        let (args, before) = lhs.split_once(" | ").expect("`<args> | <before>`");
        let (eax, after) = rhs.split_once(" | ").expect("`<eax> | <after>`");
        let a: Vec<i32> = args.split_whitespace().map(|s| s.parse().expect("an integer")).collect();
        let mut wins: Vec<Vec<u8>> = before.trim().split(';').map(unhex).collect();
        let mut m = wins.remove(0);
        let mut p: Vec<Vec<u8>> = wins.split_off(2);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0..=5 the single answers of stubs 0..=5 (unused), 6 and 7 what `center_of_gravity` writes through its
                // arguments 0 and 1 (arg 0 pairs with the `+0x10` field), then 12 values a call in call order: 8.. `get_unit`'s,
                // 20.. slot +0x8's, 32.. slot +0xbc's, 44.. slot +0x14's. Windows: `c0` the world (width at +0), `p1..p4` the
                // unit records (`+0xdc` the order list, written `+0xcc`/`+0xd0`/`+0xd4`), `p6..p9` the objects (y at +0x10,
                // x at +0x14, both xor 0x63637), `p10..p13` the lists (`+4` the node), `p14..p17` the nodes (`+8` the
                // order, `+0xc` a byte), `p22` the regions (stride 0x1c, short at +4), `p23` the `div_3_table`.
                // `vector_dist` is the crate's own, the original's being stub 5.
                use crate::army_is_moving_view::{ArmyIsMovingView, army_is_moving};
                use crate::world::{Pos, vector_dist};
                struct V<'a> {
                    a: &'a [i32],
                    m: &'a [u8],
                    c: &'a [Vec<u8>],
                    p: &'a mut [Vec<u8>],
                    k: [usize; 4],
                }
                fn rd(w: &[u8], o: usize) -> i32 {
                    i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"))
                }
                const KEY: i32 = 0x63637;
                impl V<'_> {
                    fn nth(&mut self, s: usize) -> i32 {
                        let v = self.a[8 + 12 * s + self.k[s].min(11)];
                        self.k[s] += 1;
                        v
                    }
                    fn xy(&self, u: i32) -> (i32, i32) {
                        let o = &self.p[6 + u as usize];
                        (rd(o, 0x14) ^ KEY, rd(o, 0x10) ^ KEY)
                    }
                }
                impl ArmyIsMovingView for V<'_> {
                    fn count(&self) -> i32 {
                        rd(self.m, 0x10)
                    }
                    fn unit(&mut self, _i: i32) -> i32 {
                        self.nth(0)
                    }
                    fn is_active(&mut self, _u: i32) -> bool {
                        self.nth(1) != 0
                    }
                    fn is_on_map(&mut self, _u: i32) -> bool {
                        self.nth(2) != 0
                    }
                    fn head_is_move(&mut self, u: i32) -> Option<bool> {
                        let u = u as usize;
                        if rd(&self.p[1 + u], 0xdc) == 0 {
                            return None;
                        }
                        let node = rd(&self.p[10 + u], 4);
                        let order = rd(&self.p[14 + u], 8);
                        let flag = self.p[14 + u][0xc];
                        let rec = &mut self.p[1 + u];
                        rec[0xd4..0xd8].copy_from_slice(&node.to_le_bytes());
                        rec[0xcc..0xd0].copy_from_slice(&order.to_le_bytes());
                        rec[0xd0] = flag;
                        if order == 0 {
                            return None;
                        }
                        Some(self.nth(3) != 0)
                    }
                    fn outside_region(&mut self, u: i32) -> bool {
                        let (x, y) = self.xy(u);
                        let d3 = |v: i32| rd(&self.p[23], 4 * (v >> 8) as usize);
                        let idx = d3(x)
                            .wrapping_mul(rd(&self.c[0], 0))
                            .wrapping_add(d3(y));
                        let off = (idx as usize).wrapping_mul(0x1c) + 4;
                        let reg = i16::from_le_bytes([self.p[22][off], self.p[22][off + 1]]);
                        i32::from(reg) != rd(self.m, 8)
                    }
                    fn centre_of_gravity(&mut self) -> Pos {
                        Pos::new(self.a[7] * 0x300 + 0x180, self.a[6] * 0x300 + 0x180)
                    }
                    fn dist_to(&mut self, u: i32, c: Pos) -> i32 {
                        let (x, y) = self.xy(u);
                        vector_dist((x - c.x).wrapping_abs(), (y - c.y).wrapping_abs())
                    }
                }
                let mut v = V {
                    a,
                    m,
                    c,
                    p,
                    k: [0; 4],
                };
                i64::from(army_is_moving(&mut v))
            }
        };
        assert_eq!(got, want_eax, "is_moving — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_moving — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_moving — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[3 + i], "is_moving — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Army::is_moving@006f5470: {n} rows agree");
}
