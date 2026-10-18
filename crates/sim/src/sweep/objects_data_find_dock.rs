//! `ObjectsData::find_dock@0065cfd0` under the emulator against the port (item 3120; tools/sweep_gen.py, obj with windows).

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

/// `ObjectsData::find_dock@0065cfd0`, run by `tools/emu/sweep_objects_data_find_dock.py` under unicorn
/// (tools/emu): 520 random bytes of object, 9 argument(s), `p` windows in order: p0..p15 sparse table 0 (8 rows × 2 slice(s), row-major), p16..p57 the place(s), in flag order; the global cell 0x00c061d0 points at 320 random bytes; callee 0x0067daa0 (2 args) stubbed; callee 0x0067dbb0 (4 args) stubbed; callee 0x0046cff0 (0 args) run, not hooked; the word at `0xc0a590` points at 12 random bytes less `0` (placed); the word at `0xc0a5ac` points at 12 random bytes less `0` (placed); the word at `0xc0a5c8` points at 12 random bytes less `0` (placed); the word at `0xc0a5e4` points at 12 random bytes less `0` (placed); the word at `0xc0a600` points at 12 random bytes less `0` (placed); the word at `p16+0` points at 16 random bytes less `0` (placed); the word at `p16+4` points at 16 random bytes less `0` (placed); the word at `p16+8` points at 16 random bytes less `0` (placed); the word at `p17+0` points at 16 random bytes less `0` (placed); the word at `p17+4` points at 16 random bytes less `0` (placed); the word at `p17+8` points at 16 random bytes less `0` (placed); the word at `p18+0` points at 16 random bytes less `0` (placed); the word at `p18+4` points at 16 random bytes less `0` (placed); the word at `p18+8` points at 16 random bytes less `0` (placed); the word at `p19+0` points at 16 random bytes less `0` (placed); the word at `p19+4` points at 16 random bytes less `0` (placed); the word at `p19+8` points at 16 random bytes less `0` (placed); the word at `p20+0` points at 16 random bytes less `0` (placed); the word at `p20+4` points at 16 random bytes less `0` (placed); the word at `p20+8` points at 16 random bytes less `0` (placed); the word at `0xc0ab84` points at 12 random bytes less `0` (placed); the word at `0xc0aba0` points at 12 random bytes less `0` (placed); the word at `0xc0abbc` points at 12 random bytes less `0` (placed); the word at `0xc0abd8` points at 12 random bytes less `0` (placed); the word at `0xc0abf4` points at 12 random bytes less `0` (placed); the word at `p36+0` points at 24 random bytes less `0` (placed); the word at `p36+4` points at 24 random bytes less `0` (placed); the word at `p36+8` points at 24 random bytes less `0` (placed); the word at `p37+0` points at 24 random bytes less `0` (placed); the word at `p37+4` points at 24 random bytes less `0` (placed); the word at `p37+8` points at 24 random bytes less `0` (placed); the word at `p38+0` points at 24 random bytes less `0` (placed); the word at `p38+4` points at 24 random bytes less `0` (placed); the word at `p38+8` points at 24 random bytes less `0` (placed); the word at `p39+0` points at 24 random bytes less `0` (placed); the word at `p39+4` points at 24 random bytes less `0` (placed); the word at `p39+8` points at 24 random bytes less `0` (placed); the word at `p40+0` points at 24 random bytes less `0` (placed); the word at `p40+4` points at 24 random bytes less `0` (placed); the word at `p40+8` points at 24 random bytes less `0` (placed); the word at `0xcae5fc` points at 16 random bytes less `0` (placed); the word at `c0+0x134` points at 256 random bytes less `0` (placed); stub 0 answered from [0, 1, 1, 1] (90%); stub 1 answered from [0, 1, 1, 1] (90%); stub 0 answers 6 values in call order, printed after the answers; stub 1 answers 8 values in call order, printed after the answers; 8 rows 0x6eec apart at `0xc061e0`, slices +0x0:0x4, +0x42c:0x4 (`p` windows after the pointed buffers, before the places); `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 1200 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_objects_data_find_dock.py") else {
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
    // `a`: 0..=8 the nine stack arguments (x, y, search, who, max dist, flags, filter, two for the filter), 9..=11 the stubs' singles
    // (unused), 12..=15 `valid_search`'s run, 16..=21 `valid_filter`'s. Windows: c0 the world record (width at +0), p[2r] and
    // p[2r+1] leader r's flag word and dock count, p19+3r+j its docks, p31+3r+idx the object a dock's index names, p40 the
    // `div_3_table`, p41 the world's cells. `vector_dist` is the original's own code (stub 2).
    let rd = |w: &[u8], o: usize| i32::from_le_bytes(w[o..o + 4].try_into().expect("four bytes"));
    let rd16 = |w: &[u8], o: usize| i16::from_le_bytes(w[o..o + 2].try_into().expect("two bytes")) as i32;
    const KEY: i32 = 0x63637;
    let (x, y) = (a[0], a[1]);
    let by_region = a[5] & 0x200 != 0;
    let mut region = -1;
    if by_region {
        let div3 = |v: i32| rd(&p[56], 4 * (v >> 8) as usize);
        let cell = div3(y).wrapping_mul(rd(&c[0], 0)).wrapping_add(div3(x));
        region = rd16(&p[57], 4 + 0x1c * cell as usize);
    }
    let mut who = a[3];
    let mut best = 99_999_999;
    let mut found = -1;
    let (mut k0, mut k1) = (0usize, 0usize);
    for r in 0..8usize {
        if p[2 * r][0] & 1 == 0 {
            continue;
        }
        let s = a[12 + k0.min(5)];
        k0 += 1;
        if s == 0 {
            continue;
        }
        for j in 0..rd(&p[2 * r + 1], 0).max(0) as usize {
            let dock = &p[21 + 3 * r + j];
            if dock[8] & 1 == 0 {
                continue;
            }
            let idx = rd16(dock, 2);
            if by_region && region != rd16(dock, 4) {
                continue;
            }
            let o = &p[41 + 3 * r + idx as usize];
            let d = crate::world::vector_dist(
                (rd(o, 0x10) ^ KEY).wrapping_sub(x).wrapping_abs(),
                (rd(o, 0x14) ^ KEY).wrapping_sub(y).wrapping_abs(),
            );
            if d > best || (a[4] >= 0 && d > a[4]) {
                continue;
            }
            if a[6] != 0 && ((a[6] - 1) as u32) <= 0x16 {
                let f = a[18 + k1.min(7)];
                k1 += 1;
                if f == 0 {
                    continue;
                }
            }
            best = d;
            who = r as i32;
            found = j as i32;
        }
    }
    m[0x1fc..0x200].copy_from_slice(&best.to_le_bytes());
    m[0x200..0x204].copy_from_slice(&who.to_le_bytes());
    i64::from(found)
}
        };
        assert_eq!(got, want_eax, "find_dock — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "find_dock — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "find_dock — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[2 + i], "find_dock — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 1200, "only {n} rows");
    eprintln!("ObjectsData::find_dock@0065cfd0: {n} rows agree");
}
