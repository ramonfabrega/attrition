//! `Unit::update_armor@006054c0` under the emulator against the port (item 3058; tools/sweep_gen.py, obj with windows).

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

/// `Unit::update_armor@006054c0`, run by `tools/emu/sweep_unit_update_armor.py` under unicorn
/// (tools/emu): 160 random bytes of object, 0 argument(s), `p` windows in order: p0..p30 the place(s), in flag order; the global cell 0x00c0618c points at 96 random bytes; the global cell 0x00c061f0 points at 2432 random bytes; 56792 random bytes at 0x00e3a390; vtable slot +0xa8 (0 args) stubbed; vtable slot +0xe8 (0 args) stubbed; callee 0x00647db0 (0 args) stubbed; the word at `c0+0x14` points at 24 random bytes less `0` (placed); the word at `p0+0` points at 16 random bytes less `0` (placed); the word at `p0+4` points at 16 random bytes less `0` (placed); the word at `p0+8` points at 16 random bytes less `0` (placed); the word at `p0+12` points at 16 random bytes less `0` (placed); the word at `p0+16` points at 16 random bytes less `0` (placed); the word at `p0+20` points at 16 random bytes less `0` (placed); the word at `m+0x60` points at 24 random bytes less `0` (placed); the word at `p7+0` points at 160 random bytes less `0` (placed); the word at `p7+4` points at 160 random bytes less `0` (placed); the word at `p7+8` points at 160 random bytes less `0` (placed); the word at `p7+12` points at 160 random bytes less `0` (placed); the word at `p7+16` points at 160 random bytes less `0` (placed); the word at `p7+20` points at 160 random bytes less `0` (placed); the word at `0xc0aec0` points at 16 random bytes less `0` (placed); the word at `p14+0` points at 160 random bytes less `0` (placed); the word at `p14+4` points at 160 random bytes less `0` (placed); the word at `p14+8` points at 160 random bytes less `0` (placed); the word at `p14+12` points at 160 random bytes less `0` (placed); the word at `0xc0aedc` points at 16 random bytes less `0` (placed); the word at `p19+0` points at 160 random bytes less `0` (placed); the word at `p19+4` points at 160 random bytes less `0` (placed); the word at `p19+8` points at 160 random bytes less `0` (placed); the word at `p19+12` points at 160 random bytes less `0` (placed); the word at `m+0x18` points at 8 random bytes less `0` (placed); the word at `p8+0x18` points at 8 random bytes less `0` (placed); the word at `p9+0x18` points at 8 random bytes less `0` (placed); the word at `p10+0x18` points at 8 random bytes less `0` (placed); the word at `p11+0x18` points at 8 random bytes less `0` (placed); the word at `p12+0x18` points at 8 random bytes less `0` (placed); the word at `p13+0x18` points at 8 random bytes less `0` (placed); slot +0xe8 laid as 0x0046ceb0 or its stub, a flag after the answers; the word at `c0+0x30` laid as the address of `p0` every row; stub 0 answers `p8+j`, the data of the object its `ecx` is in `p1+j` (j < 6), linked at `+0x8e`; stub 1 answers 1 values in call order, the last from [1], printed after the answers; figures p15..p18 link at +0x90 (2 bytes) above their own index or to -1; figures p20..p23 link at +0x90 (2 bytes) above their own index or to -1; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p8+0` as well as at `this`; the stubbed vtable laid at `p9+0` as well as at `this`; the stubbed vtable laid at `p10+0` as well as at `this`; the stubbed vtable laid at `p11+0` as well as at `this`; the stubbed vtable laid at `p12+0` as well as at `this`; the stubbed vtable laid at `p13+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_unit_update_armor.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(3);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // The climb: a unit whose slot +0xe8 is `is_captain` (a[4] = 1) is a captain when the high bit
                // of the short at +0x8e is set; otherwise the stub answers a[3]. A non-captain asks the object
                // `p[1 + idx]` (idx = that short) for its parent, `p[8 + idx]`. Unit 0 is `m`, unit 1 + k is `p[8 + k]`.
                let base = a[4] != 0;
                let s16 = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
                let mut cur: Option<usize> = None;
                loop {
                    let w: &[u8] = match cur {
                        None => &m[..],
                        Some(k) => &p[8 + k][..],
                    };
                    let cap = if base { u32::from(s16(w, 0x8e) as u16 >> 15) } else { a[3] as u32 };
                    if cap != 0 {
                        break;
                    }
                    cur = Some(usize::try_from(s16(w, 0x8e)).expect("index"));
                }
                let (w, ty): (Vec<u8>, i32) = match cur {
                    None => (m.to_vec(), rd(&p[24], 4)),
                    Some(k) => (p[8 + k].clone(), rd(&p[25 + k], 4)),
                };
                let b = usize::from(w[9]);
                let mut armor = a[2] as i16;
                if ty == 0x32 || ty == 0x33 {
                    let base_off = b * 0x6eec;
                    if c[2][base_off + 0x6da7] & 0x80 != 0 || c[2][base_off + 0x6dcf] & 0x80 != 0 {
                        armor = armor.wrapping_add(rd(&c[1], 0x960) as i16);
                    }
                }
                let ab = armor.to_le_bytes();
                match cur {
                    None => m[0x9c..0x9e].copy_from_slice(&ab),
                    Some(k) => p[8 + k][0x9c..0x9e].copy_from_slice(&ab),
                }
                let mut idx = s16(&w, 0x90);
                while idx >= 0 {
                    let rec = &mut p[(if b == 0 { 15 } else { 20 }) + idx as usize];
                    rec[0x9c..0x9e].copy_from_slice(&ab);
                    idx = s16(rec, 0x90);
                }
                i64::from(armor)
            }
        };
        assert_eq!(got, want_eax, "update_armor — eax, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "update_armor — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "update_armor — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[4 + i], "update_armor — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Unit::update_armor@006054c0: {n} rows agree");
}
