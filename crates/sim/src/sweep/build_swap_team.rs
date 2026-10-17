//! `Build::swap_team@00628210` under the emulator against the port (item 3058; tools/sweep_gen.py, obj with windows).

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

/// `Build::swap_team@00628210`, run by `tools/emu/sweep_build_swap_team.py` under unicorn
/// (tools/emu): 192 random bytes of object, 1 argument(s), `p` windows in order: p0..p12 the place(s), in flag order; the global cell 0x00c0618c points at 96 random bytes; the global cell 0x00c0a914 points at 1024 random bytes; the global cell 0x00c06210 points at 672 random bytes; 56792 random bytes at 0x00e3a390; 1 random bytes at 0x00e8a794; vtable slot +0x60 (2 args) stubbed; vtable slot +0xb8 (2 args) stubbed; vtable slot +0x90 (0 args) stubbed; vtable slot +0x94 (0 args) stubbed; callee 0x00645550 (0 args) stubbed; callee 0x0064cd20 (4 args) stubbed; callee 0x00620b60 (1 args) stubbed; callee 0x00640c00 (1 args) stubbed; callee 0x00643070 (1 args) stubbed; vtable slot +0x0 (1 args) stubbed; the word at `c0+0x14` points at 20 random bytes less `0` (placed); the word at `p0+0` points at 224 random bytes less `0` (placed); the word at `p0+4` points at 224 random bytes less `0` (placed); the word at `p0+8` points at 224 random bytes less `0` (placed); the word at `p0+12` points at 224 random bytes less `0` (placed); the word at `p0+16` points at 224 random bytes less `0` (placed); the word at `m+0x18` points at 16 random bytes less `0` (placed); the word at `m+0xa8` points at 48 random bytes less `0` (placed); the word at `p1+0xa8` points at 64 random bytes less `0` (placed); the word at `p2+0xa8` points at 64 random bytes less `0` (placed); the word at `p3+0xa8` points at 64 random bytes less `0` (placed); the word at `p4+0xa8` points at 64 random bytes less `0` (placed); the word at `p5+0xa8` points at 64 random bytes less `0` (placed); stub 0 answered from [0, 1] (90%); stub 1 answered from [0, 1] (90%); stub 2 answered from [0, 1] (90%); stub 3 answered from [0, 1] (90%); the word at `m+0` laid as the image vtable at 0xb42174 (the stub slots copied there) half the rows; the word at `c0+0x30` laid as the address of `p0` every row; stub 0 answers 3 values in call order, printed after the answers; stub 1 answers 3 values in call order, printed after the answers; stub 7 answers -1..4 every row; stub 4 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 5 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 6 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 7 records its `ecx` and arguments, a window's pointer by its name, after ` # `; stub 8 records its `ecx` and arguments, a window's pointer by its name, after ` # `; the stubbed vtable laid at `p1+0x98` as well as at `this`; the stubbed vtable laid at `p2+0x98` as well as at `this`; the stubbed vtable laid at `p3+0x98` as well as at `this`; the stubbed vtable laid at `p4+0x98` as well as at `this`; the stubbed vtable laid at `p5+0x98` as well as at `this`; the stubbed vtable laid at `p6+0` as well as at `this`; the stubbed vtable laid at `p1+0` as well as at `this`; the stubbed vtable laid at `p2+0` as well as at `this`; the stubbed vtable laid at `p3+0` as well as at `this`; the stubbed vtable laid at `p4+0` as well as at `this`; the stubbed vtable laid at `p5+0` as well as at `this`; `eax` and every byte of every window after held to the port (`c` the cells and tables, `p` the pointed buffers, the stubs' answers after the arguments). 800 rows (an arm few rows reach).
#[test]
fn the_emulated_original_agrees_on_every_row() {
    let Some(rows) = super::emu_rows("sweep_build_swap_team.py") else {
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
        let mut p: Vec<Vec<u8>> = wins.split_off(5);
        let mut c: Vec<Vec<u8>> = wins;
        let want_eax: i64 = eax.trim().parse().expect("an integer");
        let mut log: Vec<String> = Vec::new();
        let got: i64 = {
            let (a, m, c, p) = (&a[..], &mut m[..], &mut c[..], &mut p[..]);
            let _ = (&a, &m, &c, &p);
            {
                // `a`: 0 the player, 1..=10 the stubs' answers (slot 0x90 is a[3], slot 0x94 a[4], Wall::swap_team a[8]),
                // 11..14 slot +0x60's three answers (this's vtable is `Build::vftable`), 14..17 slot +0xb8's (it is not).
                let param = a[0];
                let base = rd(m, 0) == 0xb4_2174;
                let site = |k: usize| if base { a[11 + k] } else { a[14 + k] };
                let s16 = |w: &[u8], o: usize| i16::from_le_bytes([w[o], w[o + 1]]);
                let mut l4 = "-".to_string();
                let mut l6 = "-".to_string();
                let mut l8 = "-".to_string();
                if site(0) != 0 {
                    l4 = "m+0x0".to_string();
                }
                let l5 = "m+0x0,1,-1,0,1".to_string();
                if site(1) == 0 {
                    l6 = "m+0x0,1".to_string();
                }
                let l7 = format!("m+0x0,{param}");
                let h = a[8];
                if h >= 0 {
                    let o = 1 + h as usize;
                    let d = 8 + h as usize;
                    p[o][0x84] = m[0x84];
                    p[o][0x85] = m[0x85];
                    let (b7a, b7b) = (m[0x7a], m[0x7b]);
                    p[o][0x7a] = b7a;
                    p[o][0x7b] = b7b;
                    p[o][0x7f] = m[0x7f];
                    p[o][0x83] = p[o][0x83].max(m[0x83]);
                    let mut ecx = "p6+0x0".to_string();
                    if a[3] != 0 && a[4] == 0 {
                        let n = rd(m, 0x9c);
                        for i in 0..n.max(0) as usize {
                            let src = p[7][i * 8..i * 8 + 8].to_vec();
                            let cnt = rd(&p[o], 0x9c);
                            p[d][cnt as usize * 8..cnt as usize * 8 + 8].copy_from_slice(&src);
                            wr(&mut p[o], 0x9c, cnt + 1);
                            ecx = cnt.to_string();
                        }
                        p[o][0xb4] = m[0xb4];
                        p[o][0xb5] = m[0xb5];
                        m[0xb4] = 0xff;
                        m[0xb5] = 0xff;
                        wr(m, 0x9c, 0);
                    }
                    if rd(&p[6], 4) == 0x1a1 {
                        p[o][0x78] = m[0x78];
                        p[o][0x79] = m[0x79];
                        let idx = s16(m, 0x78) as isize as usize;
                        wr(&mut c[1], idx * 0xc0, param);
                        wr(&mut c[1], idx * 0xc0 + 4, h);
                        m[0x78] = 0xff;
                        m[0x79] = 0xff;
                        ecx = (idx * 0xc0).to_string();
                    }
                    l8 = format!("p{o}+0x0,{ecx}");
                }
                if site(2) != 0 {
                    let off = param as usize * 0x6eec;
                    let v = rd(&c[3], off) | 0x800;
                    wr(&mut c[3], off, v);
                    if param == rd(&c[2], 0x298) {
                        c[4][0] |= 2;
                    }
                }
                log.extend([l4, l5, l6, l7, l8]);
                i64::from(h)
            }
        };
        assert_eq!(got, want_eax, "swap_team — eax, row `{line}`");
        assert_eq!(log, want_log, "swap_team — the recorded calls, row `{line}`");
        let want: Vec<Vec<u8>> = after.trim().split(';').map(unhex).collect();
        assert_eq!(m, want[0], "swap_team — this's bytes, row `{line}`");
        for (i, w) in c.iter().enumerate() {
            assert_eq!(w, &want[1 + i], "swap_team — window c[{i}], row `{line}`");
        }
        for (i, w) in p.iter().enumerate() {
            assert_eq!(w, &want[6 + i], "swap_team — window p[{i}], row `{line}`");
        }
        n += 1;
    }
    assert!(n >= 800, "only {n} rows");
    eprintln!("Build::swap_team@00628210: {n} rows agree");
}
