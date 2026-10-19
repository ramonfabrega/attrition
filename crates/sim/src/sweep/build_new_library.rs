//! `Build::new_library@00627fe0` under the emulator against the port (item 3304; tools/sweep_gen.py, obj with windows).

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

/// `Build::new_library@00627fe0`, run by `tools/emu/sweep_build_new_library.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p18 the place(s), in flag order; vtable slot +0x60 (0 args) stubbed; vtable slot +0xb8 (2 args) stubbed; callee 0x006db6c0 (0 args) stubbed; the word at `0xc0ab84+2*0x1c` points at 32 random bytes less `0` (placed); the word at `p0+0` points at 152 random bytes less `0` (placed); the word at `p0+4` points at 152 random bytes less `0` (placed); the word at `p0+8` points at 152 random bytes less `0` (placed); the word at `p0+12` points at 152 random bytes less `0` (placed); the word at `p0+16` points at 152 random bytes less `0` (placed); the word at `p0+20` points at 152 random bytes less `0` (placed); the word at `p0+24` points at 152 random bytes less `0` (placed); the word at `p0+28` points at 152 random bytes less `0` (placed); the word at `p1+0x8c` points at 80 random bytes less `0` (placed); the word at `p2+0x8c` points at 80 random bytes less `0` (placed); the word at `p3+0x8c` points at 80 random bytes less `0` (placed); the word at `p4+0x8c` points at 80 random bytes less `0` (placed); the word at `p5+0x8c` points at 80 random bytes less `0` (placed); the word at `p6+0x8c` points at 80 random bytes less `0` (placed); the word at `p7+0x8c` points at 80 random bytes less `0` (placed); the word at `p8+0x8c` points at 80 random bytes less `0` (placed); the word at `m+0x8c` points at 80 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; stub 2 answers -1..7 every row; the stubbed vtable laid at `p18+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 600 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_new_library.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(0);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            // The crate's `new_library_steps` over the byte windows: this is `m`, the owner's
            // object table `p[1 + k]` (object k, its queue entries behind `p[9 + k]`), this's
            // own entries `p[17]`. Stubs `a[0]` (this's slot `+0x60` when `*this` is the Build
            // vtable) or `a[1]` (slot `+0xb8`) answer `is`; `a[2]` is the first library.
            struct V<'a> {
                a: &'a [i32],
                m: &'a mut [u8],
                p: &'a mut [Vec<u8>],
            }
            impl crate::city::NewLibraryView for V<'_> {
                fn is_library(&mut self) -> bool {
                    (if rd(self.m, 0) == 0x00b4_2174 {
                        self.a[0]
                    } else {
                        self.a[1]
                    }) != 0
                }
                fn first_library(&mut self) -> i32 {
                    self.a[2]
                }
                fn own_index(&mut self) -> i32 {
                    i32::from(i16::from_le_bytes([self.m[0xa], self.m[0xb]]))
                }
                fn source_queued(&mut self, first: i32) -> u8 {
                    self.p[1 + first as usize][0x82]
                }
                fn copy_entry(&mut self, first: i32, i: usize) {
                    let src: Vec<u8> =
                        self.p[9 + first as usize][i * 0x14..(i + 1) * 0x14].to_vec();
                    self.p[17][i * 0x14..(i + 1) * 0x14].copy_from_slice(&src);
                }
                fn set_queued(&mut self, n: u8) {
                    self.m[0x82] = n;
                }
                fn clear_source(&mut self, first: i32) {
                    self.p[1 + first as usize][0x82] = 0;
                }
            }
            crate::city::new_library_steps(&mut V { a, m, p });
            want_eax
        };
        let _ = (got, want_eax);
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "new_library — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "new_library — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "new_library — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 600, "only {n} rows");
    eprintln!("Build::new_library@00627fe0: {n} rows agree");
}
