//! `Army::is_engaged@006f56d0` under the emulator against the port (item 3112; tools/sweep_gen.py, obj with windows).

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

/// `Army::is_engaged@006f56d0`, run by `tools/emu/sweep_army_is_engaged.py` under unicorn
/// (tools/emu): 152 random bytes of object, 0 argument(s), `p` windows in order: p0..p9 the place(s), in flag order, p10 stub 4's answer window; the global cell 0x00c0618c points at 64 random bytes; callee 0x006f9b50 (0 args) stubbed; callee 0x006f9df0 (1 args) stubbed; vtable slot +0x8 (0 args) stubbed; vtable slot +0xbc (0 args) stubbed; callee 0x00608450 (0 args) stubbed; vtable slot +0x10 (0 args) stubbed; callee 0x006f8a20 (2 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p0+0` points at 32 random bytes less `0` (placed); the word at `p0+4` points at 32 random bytes less `0` (placed); the word at `p0+8` points at 32 random bytes less `0` (placed); the word at `p0+12` points at 32 random bytes less `0` (placed); the word at `c0+0x14` points at 16 random bytes less `0` (placed); the word at `p5+0` points at 32 random bytes less `0` (placed); the word at `p5+4` points at 32 random bytes less `0` (placed); the word at `p5+8` points at 32 random bytes less `0` (placed); the word at `p5+12` points at 32 random bytes less `0` (placed); stub 5 answered from [10, 10, 9] (90%); stub 4 answers the address of 32 random bytes, a `p` window after the places; stub 1 answers 4 values in call order, printed after the answers; stub 2 answers 4 values in call order, printed after the answers; stub 3 answers 4 values in call order, printed after the answers; stub 5 answers 4 values in call order, printed after the answers; stub 1 answers 0..3 every row; stub 2 answers 0..1 every row; stub 3 answers 0..1 every row; stub 6 writes a value in [0, 1, 2, 3] through its argument 0, printed after the answers (`o` in a place); stub 6 writes a value in [0, 1, 2, 3] through its argument 1, printed after the answers (`o` in a place); the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 400 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_army_is_engaged.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(1);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0..=7 the single answers of stubs 0..=7 (only stub 4's, the action, is used: p10), 8 and 9 what
                // `center_of_gravity` writes through its arguments 0 and 1, 10..=13 `get_unit`'s run, 14..=17 slot +0x8's,
                // 18..=21 slot +0xbc's, 22..=25 slot +0x10's. Windows: p6..p9 the objects' records (x at +0x14, y at +0x10, both
                // xor 0x63637). `vector_dist` is the original's own code (stub 7).
                let rd = |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
                const KEY: i32 = 0x63637;
                let count = rd(m, 0x10);
                let (mut k1, mut k2, mut k3, mut k5) = (0usize, 0usize, 0usize, 0usize);
                let mut hits = 0i32;
                for _ in 0..count.max(0) {
                    let u = a[10 + k1.min(3)] as usize;
                    k1 += 1;
                    let s8 = a[14 + k2.min(3)];
                    k2 += 1;
                    if s8 == 0 {
                        continue;
                    }
                    let sbc = a[18 + k3.min(3)];
                    k3 += 1;
                    if sbc == 0 {
                        continue;
                    }
                    let ord = a[22 + k5.min(3)];
                    k5 += 1;
                    if ord != 10 {
                        continue;
                    }
                    let r = &p[6 + u];
                    let (ry, rx) = (rd(r, 0x10) ^ KEY, rd(r, 0x14) ^ KEY);
                    let d = crate::world::vector_dist(
                        ry.wrapping_sub(rd(m, 0x38)).wrapping_abs(),
                        rx.wrapping_sub(rd(m, 0x3c)).wrapping_abs(),
                    );
                    if d >= 0xc00 {
                        continue;
                    }
                    let d2 = crate::world::vector_dist(
                        a[8].wrapping_mul(0x300).wrapping_sub(ry).wrapping_add(0x180).wrapping_abs(),
                        a[9].wrapping_mul(0x300).wrapping_sub(rx).wrapping_add(0x180).wrapping_abs(),
                    );
                    if d2 < 0xc01 {
                        hits += 1;
                    }
                }
                i64::from(hits != 0 && ((count + ((count >> 31) & 3)) >> 2) < hits)
            }
        };
        assert_eq!(got, want_eax, "is_engaged — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "is_engaged — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "is_engaged — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "is_engaged — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 400, "only {n} rows");
    eprintln!("Army::is_engaged@006f56d0: {n} rows agree");
}
