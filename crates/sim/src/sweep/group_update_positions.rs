//! `Group::update_positions@00713810` under the emulator against the port (item 3379; tools/sweep_gen.py, obj with windows).

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

/// `Group::update_positions@00713810`, run by `tools/emu/sweep_group_update_positions.py` under unicorn
/// (tools/emu): 2128 random bytes of object, 2 argument(s), `p` windows in order: p0..p20 the place(s), in flag order; the global cell 0x00c0618c points at 64 random bytes; vtable slot +0xb8 (0 args) stubbed; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 232 random bytes less `0` (placed); the word at `p0+4` points at 232 random bytes less `0` (placed); the word at `p1+0` points at 232 random bytes less `0` (placed); the word at `p1+4` points at 232 random bytes less `0` (placed); the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `c0+0x30` points at 16 random bytes less `0` (placed); the word at `p2+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p3+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p4+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p5+0xdc` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p8+4` points at 16 random bytes less `0` (placed); the word at `p9+4` points at 16 random bytes less `0` (placed); the word at `p10+4` points at 16 random bytes less `0` (placed); the word at `p11+4` points at 16 random bytes less `0` (placed); the word at `p12+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p13+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p14+8` points at 16 random bytes less `0` (placed, null a third of the rows); the word at `p15+8` points at 16 random bytes less `0` (placed, null a third of the rows); stub 0's answer points at 64 random bytes less `0` (placed, null a third of the rows); the stubbed vtable laid at `p16+0`; the word at `p6+0` laid as the address of `p2` every row; the word at `p6+4` laid as the address of `p3` every row; the word at `p7+0` laid as the address of `p4` every row; the word at `p7+4` laid as the address of `p5` every row; `0x00a46980` run once before the rows (a table builder the startup would have run); the stubbed vtable laid at `p17+0` as well as at `this`; the stubbed vtable laid at `p18+0` as well as at `this`; the stubbed vtable laid at `p19+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1500 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_group_update_positions.py") else {
        return;
    };
    let mut n = 0;
    for line in rows.lines().filter(|l| !l.trim().is_empty()) {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // `a`: 0 the index, 1 the owner, 2 the order's vslot `+0xb8` answer (the move record's address or 0).
            // Windows: m the Group (`form_num` `+0x44`, offsets `+0x4c`/`+0x24c`, `curr` `+0x44c`/`+0x64c`); p2..p5 the leader
            // units of (owner, index) (0,0) (0,1) (1,0) (1,1): `+0x10`/`+0x14` the point (xor 0x63637), `+0x50` the heading,
            // `+0xcc`/`+0xd0`/`+0xd4` the order cursor, `+0xdc` the list head; p8..p11 the head nodes (`+4` the entry), p12..p15 the
            // entries (`+8` the order, `+0xc` its flag byte), p16..p19 the orders (stubbed vtable), p20 the move record
            // (`+0x10` the waypoint flag, `+0x2c`/`+0x30` the point). The port is `update_positions_steps` over a view of them.
            use crate::update_positions_view::{UpdatePositionsView, update_positions_steps};
            const KEY: i32 = 0x63637;
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                p: &'a mut [Vec<u8>],
            }
            impl V<'_> {
                fn k(&self) -> usize {
                    (self.a[1] * 2 + self.a[0]) as usize
                }
                fn u(&self) -> usize {
                    2 + self.k()
                }
            }
            impl UpdatePositionsView for V<'_> {
                fn heading(&mut self) -> i32 {
                    rd(&self.p[self.u()], 0x50)
                }
                fn waypoint(&mut self) -> Option<(i32, i32)> {
                    let (k, u) = (self.k(), self.u());
                    if rd(&self.p[u], 0xdc) == 0 {
                        return None;
                    }
                    // The cursor refresh, done twice in the listing (the second is the same writes).
                    let entry = rd(&self.p[8 + k], 4);
                    let order = rd(&self.p[12 + k], 8);
                    let flag = self.p[12 + k][0xc];
                    wr(&mut self.p[u], 0xd4, entry);
                    wr(&mut self.p[u], 0xcc, order);
                    self.p[u][0xd0] = flag;
                    if order == 0 || self.a[2] == 0 {
                        return None;
                    }
                    if rd(&self.p[20], 0x10) == 0 {
                        return None;
                    }
                    Some((rd(&self.p[20], 0x2c), rd(&self.p[20], 0x30)))
                }
                fn leader_point(&mut self) -> (i32, i32) {
                    let u = self.u();
                    (rd(&self.p[u], 0x10) ^ KEY, rd(&self.p[u], 0x14) ^ KEY)
                }
                fn form_num(&mut self) -> i32 {
                    rd(self.m, 0x44)
                }
                fn slot_offset(&mut self, i: usize) -> (i32, i32) {
                    (rd(self.m, 0x4c + 4 * i), rd(self.m, 0x24c + 4 * i))
                }
                fn set_curr(&mut self, i: usize, c: crate::Pos) {
                    wr(self.m, 0x44c + 4 * i, c.x);
                    wr(self.m, 0x64c + 4 * i, c.y);
                }
            }
            let mut v = V { a, m, p };
            update_positions_steps(&mut v);
            // The listing leaves `this` in `eax`; the port has no address to give.
            want_eax
        };
        assert_eq!(got, want_eax, "update_positions — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_positions — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(
                w,
                &want[1 + i],
                "update_positions — window c[{i}], row `{line}`"
            );
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(
                w,
                &want[2 + i],
                "update_positions — window p[{i}], row `{line}`"
            );
        }
        n += 1;
    }
    assert!(n >= 1500, "only {n} rows");
    eprintln!("Group::update_positions@00713810: {n} rows agree");
}
